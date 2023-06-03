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
            if self.0.is_empty() {
                return Ok(());
            }
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
        CalScale,
        Method,
    }

    #[derive(Debug, Clone)]
    enum CalendarIteratorState {
        Preamble(PreambleState),
        StartEvent(u8),
        Event(u8, DeliveryDateIterator),
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
                        *x = PreambleState::CalScale;
                        (Some("PRODID:-//Aasan//Aasan Postgang//EN".into()), None)
                    }
                    PreambleState::CalScale => {
                        *x = PreambleState::Method;
                        (Some("CALSCALE:GREGORIAN".into()), None)
                    }
                    PreambleState::Method => (
                        Some("METHOD:PUBLISH".into()),
                        Some(CalendarIteratorState::StartEvent(0)),
                    ),
                },
                CalendarIteratorState::StartEvent(index) => {
                    if let Some(mut iterator) = self
                        .calendar
                        .delivery_dates
                        .get(usize::from(*index))
                        .map(|x| DeliveryDateIterator::new(*x, self.calendar.created))
                    {
                        (
                            iterator.next(),
                            Some(CalendarIteratorState::Event(*index, iterator)),
                        )
                    } else {
                        (
                            Some("END:VCALENDAR".into()),
                            Some(CalendarIteratorState::Done),
                        )
                    }
                }
                CalendarIteratorState::Event(index, iterator) => match iterator.next() {
                    None => (
                        Some("".into()),
                        Some(CalendarIteratorState::StartEvent(*index + 1)),
                    ),
                    x => (x, None),
                },
                CalendarIteratorState::Done => (None, None),
            };
            if let Some(s) = next_state {
                self.state = s;
            };
            res
        }
    }

    #[derive(Debug, Clone, Copy)]
    enum DeliveryDateIteratorState {
        Begin,
        DtEnd,
        DtStamp,
        DtStart,
        Summary,
        Transp,
        Uid,
        Url,
        End,
        Done,
    }

    impl DeliveryDateIteratorState {
        fn next(self) -> Option<Self> {
            use DeliveryDateIteratorState::{
                Begin, Done, DtEnd, DtStamp, DtStart, End, Summary, Transp, Uid, Url,
            };
            match self {
                Begin => Some(DtEnd),
                DtEnd => Some(DtStamp),
                DtStamp => Some(DtStart),
                DtStart => Some(Summary),
                Summary => Some(Transp),
                Transp => Some(Uid),
                Uid => Some(Url),
                Url => Some(End),
                End => Some(Done),
                Done => None,
            }
        }
    }

    #[derive(Debug, Clone)]
    struct DeliveryDateIterator {
        delivery_date: DeliveryDate,
        created: Option<DateTime<Utc>>,
        state: DeliveryDateIteratorState,
    }

    impl DeliveryDateIterator {
        #[inline]
        pub fn new(delivery_date: DeliveryDate, created: Option<DateTime<Utc>>) -> Self {
            Self {
                delivery_date,
                created,
                state: DeliveryDateIteratorState::Begin,
            }
        }
    }

    impl Iterator for DeliveryDateIterator {
        type Item = ContentLine;

        fn next(&mut self) -> Option<Self::Item> {
            use DeliveryDateIteratorState::{
                Begin, Done, DtEnd, DtStamp, DtStart, End, Summary, Transp, Uid, Url,
            };
            let value = &self.delivery_date;
            let res = match self.state {
                Begin => Some("BEGIN:VEVENT".into()),
                DtEnd => {
                    let dt_end = format_naive_date(value.date + Duration::days(1));
                    Some(format!("DTEND;VALUE=DATE:{dt_end}").into())
                }
                DtStamp => {
                    let timestamp = format_timestamp(&(self.created.unwrap_or(Utc::now())));
                    Some(format!("DTSTAMP:{timestamp}").into())
                }
                DtStart => {
                    let dt_start = format_naive_date(value.date);
                    Some(format!("DTSTART;VALUE=DATE:{dt_start}").into())
                }
                Summary => {
                    let postal_code = value.postal_code;
                    let weekday = weekday(value.date);
                    let day = value.date.day();
                    Some(format!("SUMMARY:{postal_code}: Posten kommer {weekday} {day}.").into())
                }
                Transp => Some("TRANSP:TRANSPARENT".into()),
                Uid => {
                    let date = value.date;
                    let postal_code = value.postal_code;
                    Some(format!("UID:postgang-{postal_code}-{date}").into())
                }
                Url => Some("URL:https://www.posten.no/levering-av-post/".into()),
                End => Some("END:VEVENT".into()),
                Done => None,
            };
            if let Some(s) = self.state.next() {
                self.state = s;
            }
            res
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

    #[test]
    fn test_output_line_display_empty() {
        let line = ContentLine::from("");
        assert_eq!(format!("{line}"), "");
    }
}
