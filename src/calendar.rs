//! iCalendar generator
use core::fmt;

use chrono::{
    format::{DelayedFormat, StrftimeItems},
    DateTime, Datelike, Duration, NaiveDate, Utc,
    Weekday::{Fri, Mon, Sat, Sun, Thu, Tue, Wed},
};

use crate::bring_client::mailbox_delivery_dates::DeliveryDate;

use self::content_line::ContentLine;

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
    fn content_lines(&self) -> Vec<ContentLine> {
        self.into()
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
        for content_line in self.content_lines() {
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

    #[derive(Debug)]
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

    #[derive(Debug)]
    struct DeliveryDateEntry {
        delivery_date: DeliveryDate,
        created: Option<DateTime<Utc>>,
    }

    impl From<&DeliveryDateEntry> for Vec<ContentLine> {
        fn from(value: &DeliveryDateEntry) -> Self {
            let date = value.delivery_date.date;
            let dt_end = format_naive_date(date + Duration::days(1));
            let timestamp = format_timestamp(&(value.created.unwrap_or(Utc::now())));
            let dt_start = format_naive_date(date);
            let postal_code = value.delivery_date.postal_code;
            let weekday = weekday(value.delivery_date.date);
            let day = value.delivery_date.date.day();
            vec![
                "BEGIN:VEVENT".into(),
                format!("DTEND;VALUE=DATE:{dt_end}").into(),
                format!("DTSTAMP:{timestamp}").into(),
                format!("DTSTART;VALUE=DATE:{dt_start}").into(),
                format!("SUMMARY:{postal_code}: Posten kommer {weekday} {day}.").into(),
                "TRANSP:TRANSPARENT".into(),
                format!("UID:postgang-{postal_code}-{date}").into(),
                "URL:https://www.posten.no/levering-av-post/".into(),
                "END:VEVENT".into(),
            ]
        }
    }

    impl From<&Calendar> for Vec<ContentLine> {
        fn from(value: &Calendar) -> Self {
            let mut res: Vec<ContentLine> = vec![
                "BEGIN:VCALENDAR".into(),
                "VERSION:2.0".into(),
                "PRODID:-//Aasan//Aasan Postgang//EN".into(),
                "CALSCALE:GREGORIAN".into(),
                "METHOD:PUBLISH".into(),
            ];
            res.extend(value.delivery_dates.iter().flat_map(|x| {
                let xs: Vec<ContentLine> = (&DeliveryDateEntry {
                    delivery_date: *x,
                    created: value.created,
                })
                    .into();
                xs
            }));
            res.push("END:VCALENDAR".into());
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
