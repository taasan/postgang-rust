//! iCalendar generator
use core::fmt;

use chrono::{
    format::{DelayedFormat, StrftimeItems},
    DateTime, Datelike, Duration, NaiveDate, Utc,
    Weekday::{Fri, Mon, Sat, Sun, Thu, Tue, Wed},
};

use crate::bring_client::mailbox_delivery_dates::DeliveryDate;

use self::content_line::CalendarIterator;

#[inline]
fn format_naive_date<'a>(date: NaiveDate) -> DelayedFormat<StrftimeItems<'a>> {
    date.format("%Y%m%d")
}

#[inline]
fn format_timestamp<'a>(timestamp: &DateTime<Utc>) -> DelayedFormat<StrftimeItems<'a>> {
    timestamp.format("%Y%m%dT%H%M%SZ")
}

fn weekday(date: NaiveDate) -> &'static str {
    match date.weekday() {
        Mon => "mandag",
        Tue => "tirsdag",
        Wed => "onsdag",
        Thu => "torsdag",
        Fri => "fredag",
        Sat => "lørdag",
        Sun => "søndag",
    }
}

#[derive(Debug, Clone)]
pub struct Calendar {
    delivery_dates: Vec<DeliveryDate>,
    created: Option<DateTime<Utc>>,
}

impl Calendar {
    fn iter(&self) -> CalendarIterator {
        CalendarIterator::new(self.clone())
    }
}

impl From<Vec<DeliveryDate>> for Calendar {
    fn from(value: Vec<DeliveryDate>) -> Self {
        Self::new(value, None)
    }
}

impl Calendar {
    #[must_use]
    pub fn new(delivery_dates: Vec<DeliveryDate>, created: Option<DateTime<Utc>>) -> Self {
        Self {
            delivery_dates,
            created,
        }
    }
}

impl fmt::Display for Calendar {
    /// Format [`Calendar`] as an iCalendar string.
    ///
    /// ```
    /// use chrono::{DateTime, FixedOffset, NaiveDate, Utc};
    /// use postgang::bring_client::mailbox_delivery_dates::DeliveryDate;
    /// use postgang::bring_client::NorwegianPostalCode;
    /// use postgang::calendar::Calendar;
    ///
    /// let postal_code = NorwegianPostalCode::try_from("7800").unwrap();
    /// let date = NaiveDate::from_ymd_opt(1970, 8, 13).unwrap();
    /// let created = Some(DateTime::<FixedOffset>::parse_from_rfc3339("1970-08-13T00:00:00Z").unwrap().into());
    /// let delivery_dates = vec![DeliveryDate::new(postal_code, date)];
    /// let calendar = Calendar::new(delivery_dates, created);
    /// let ical_str = calendar.to_string();
    ///
    /// assert_eq!(
    ///     ical_str,
    ///     "BEGIN:VCALENDAR\r\n\
    ///      VERSION:2.0\r\n\
    ///      PRODID:-//Aasan//Aasan Postgang//EN\r\n\
    ///      CALSCALE:GREGORIAN\r\n\
    ///      METHOD:PUBLISH\r\n\
    ///      BEGIN:VEVENT\r\n\
    ///      DTEND;VALUE=DATE:19700814\r\n\
    ///      DTSTAMP:19700813T000000Z\r\n\
    ///      DTSTART;VALUE=DATE:19700813\r\n\
    ///      SUMMARY:7800: Posten kommer torsdag 13.\r\n\
    ///      TRANSP:TRANSPARENT\r\n\
    ///      UID:postgang-7800-1970-08-13\r\n\
    ///      URL:https://www.posten.no/levering-av-post/\r\n\
    ///      END:VEVENT\r\n\
    ///      END:VCALENDAR\r\n");
    ///
    /// let calendar = Calendar::new(vec![], created);
    /// let ical_str = calendar.to_string();
    /// assert_eq!(
    ///     ical_str,
    ///     "BEGIN:VCALENDAR\r\n\
    ///      VERSION:2.0\r\n\
    ///      PRODID:-//Aasan//Aasan Postgang//EN\r\n\
    ///      CALSCALE:GREGORIAN\r\n\
    ///      METHOD:PUBLISH\r\n\
    ///      END:VCALENDAR\r\n");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for content_line in self.iter() {
            content_line.fmt(f)?;
        }
        Ok(())
    }
}

mod content_line {
    use core::fmt;

    use crate::bring_client::mailbox_delivery_dates::DeliveryDate;

    use super::{
        format_naive_date, format_timestamp, weekday, Calendar, DateTime, Datelike, Duration, Utc,
    };

    #[derive(Debug, Clone)]
    pub(super) struct ContentLine(String);

    impl From<&str> for ContentLine {
        fn from(x: &str) -> Self {
            ContentLine(x.to_string())
        }
    }

    impl From<String> for ContentLine {
        fn from(x: String) -> Self {
            ContentLine(x)
        }
    }

    impl fmt::Display for ContentLine {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let content = self.0.replace('\n', "\\n");
            let mut content = content.as_str();
            let mut boundary = next_boundary(&ContentLineToPrint::First(content));
            f.write_str(&content[..boundary])?;

            while boundary < content.len() {
                content = &content[boundary..];
                f.write_str("\r\n ")?;
                boundary = next_boundary(&ContentLineToPrint::Subsequent(content));
                f.write_str(&content[..boundary])?;
            }
            f.write_str("\r\n")
        }
    }

    #[derive(Debug, Clone)]
    enum PreambleState {
        Begin,
        Version,
        ProdId,
        Calscale,
        Method,
    }

    #[derive(Debug, Clone)]
    enum CalendarIteratorState {
        Preamble(PreambleState),
        PreambleEnd,
        Content(u8, DeliveryDateIterator),
        Done,
    }

    #[derive(Debug)]
    pub(super) struct CalendarIterator {
        calendar: Calendar,
        state: CalendarIteratorState,
    }

    impl CalendarIterator {
        #[inline]
        pub fn new(calendar: Calendar) -> Self {
            Self {
                calendar,
                state: CalendarIteratorState::Preamble(PreambleState::Begin),
            }
        }
    }
    impl Iterator for CalendarIterator {
        type Item = ContentLine;

        fn next(&mut self) -> Option<Self::Item> {
            log::trace!("{:?}", self.state);
            let (res, next_state) = match &mut self.state {
                CalendarIteratorState::Preamble(x) => match x {
                    PreambleState::Begin => {
                        *x = PreambleState::Version;
                        (Some("BEGIN:VCALENDAR".into()), None)
                    }
                    PreambleState::Version => {
                        *x = PreambleState::ProdId;
                        (Some("VERSION:2.0".into()), None)
                    }
                    PreambleState::ProdId => {
                        *x = PreambleState::Calscale;
                        (Some("PRODID:-//Aasan//Aasan Postgang//EN".into()), None)
                    }
                    PreambleState::Calscale => {
                        *x = PreambleState::Method;
                        (Some("CALSCALE:GREGORIAN".into()), None)
                    }
                    PreambleState::Method => (
                        Some("METHOD:PUBLISH".into()),
                        Some(CalendarIteratorState::PreambleEnd),
                    ),
                },
                CalendarIteratorState::PreambleEnd => {
                    if let Some(delivery_date) = self.calendar.delivery_dates.get(0) {
                        let mut iterator =
                            DeliveryDateIterator::new(*delivery_date, self.calendar.created);
                        match iterator.next() {
                            None => (
                                Some("END:VCALENDAR".into()),
                                Some(CalendarIteratorState::Done),
                            ),
                            x => (x, Some(CalendarIteratorState::Content(0, iterator))),
                        }
                    } else {
                        (
                            Some("END:VCALENDAR".into()),
                            Some(CalendarIteratorState::Done),
                        )
                    }
                }
                CalendarIteratorState::Content(index, iterator) => {
                    let index = *index + 1;
                    match iterator.next() {
                        None => {
                            if let Some(delivery_date) =
                                self.calendar.delivery_dates.get(usize::from(index))
                            {
                                let mut iterator = DeliveryDateIterator::new(
                                    *delivery_date,
                                    self.calendar.created,
                                );
                                (
                                    iterator.next(),
                                    Some(CalendarIteratorState::Content(index, iterator)),
                                )
                            } else {
                                (
                                    Some("END:VCALENDAR".into()),
                                    Some(CalendarIteratorState::Done),
                                )
                            }
                        }
                        x => (x, None),
                    }
                }
                CalendarIteratorState::Done => (None, None),
            };
            if let Some(s) = next_state {
                self.state = s;
            };
            res
        }
    }

    #[derive(Debug, Clone)]
    struct DeliveryDateIterator {
        delivery_date: DeliveryDate,
        created: Option<DateTime<Utc>>,
        line_no: u8,
    }

    impl DeliveryDateIterator {
        #[inline]
        pub fn new(delivery_date: DeliveryDate, created: Option<DateTime<Utc>>) -> Self {
            Self {
                delivery_date,
                created,
                line_no: 0,
            }
        }
    }

    impl Iterator for DeliveryDateIterator {
        type Item = ContentLine;

        fn next(&mut self) -> Option<Self::Item> {
            self.line_no += 1;
            let value = &self.delivery_date;
            let date = value.date;
            let dt_start = format_naive_date(value.date);
            let dt_end = format_naive_date(value.date + Duration::days(1));
            let postal_code = value.postal_code;
            let weekday = weekday(value.date);
            let timestamp = format_timestamp(&(self.created.unwrap_or(Utc::now())));
            let day = date.day();
            match self.line_no {
                1 => Some("BEGIN:VEVENT".into()),
                2 => Some(format!("DTEND;VALUE=DATE:{dt_end}").into()),
                3 => Some(format!("DTSTAMP:{timestamp}").into()),
                4 => Some(format!("DTSTART;VALUE=DATE:{dt_start}").into()),
                5 => Some(format!("SUMMARY:{postal_code}: Posten kommer {weekday} {day}.").into()),
                6 => Some("TRANSP:TRANSPARENT".into()),
                7 => Some(format!("UID:postgang-{postal_code}-{date}").into()),
                8 => Some("URL:https://www.posten.no/levering-av-post/".into()),
                9 => Some("END:VEVENT".into()),
                _ => None,
            }
        }
    }

    enum ContentLineToPrint<'a> {
        First(&'a str),
        Subsequent(&'a str),
    }

    fn next_boundary(content: &ContentLineToPrint) -> usize {
        const MAX_LINE: usize = 75;
        let (content, limit) = match content {
            ContentLineToPrint::First(x) => (x, MAX_LINE),
            ContentLineToPrint::Subsequent(x) => (x, MAX_LINE - 1),
        };
        let content = content.as_bytes();
        let num_bytes = content.len();
        if limit >= num_bytes {
            return num_bytes;
        }
        match content[..=limit]
            .iter()
            .rposition(|&c| !(128..192).contains(&c))
        {
            Some(0) | None => num_bytes,
            Some(i) => i,
        }
    }

    #[test]
    fn test_output_line_display_over_75() {
        let line = ContentLine::from(
            "123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789 ",
        );
        assert_eq!(format!("{line}"), String::from("123456789 123456789 123456789 123456789 123456789 123456789 123456789 12345\r\n 6789 \r\n"))
    }

    #[test]
    fn test_output_line_display_75() {
        let line = ContentLine::from(
            "123456789 123456789 123456789 123456789 123456789 123456789 123456789 12345",
        );
        assert_eq!(
            format!("{line}"),
            String::from(
                "123456789 123456789 123456789 123456789 123456789 123456789 123456789 12345\r\n"
            )
        )
    }

    #[test]
    fn test_output_line_display_wide_chars() {
        let line = ContentLine::from("A☣️☣️☣️☣️☣️☣️☣️☣️☣️☣️☣️☣️☣️☣️☣️☣️☣️☣️");
        assert_eq!(
            format!("{line}"),
            String::from("A☣️☣️☣️☣️☣️☣️☣️☣️☣️☣️☣️☣️\r\n ☣️☣️☣️☣️☣️☣️\r\n")
        )
    }

    #[test]
    fn test_output_line_display_newline() {
        let line = ContentLine::from("A\nnna");
        assert_eq!(format!("{line}"), "A\\nnna\r\n");
    }
}
