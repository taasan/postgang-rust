//! iCalendar generator
use super::bring_client::mailbox_delivery_dates::DeliveryDate;
use chrono::{
    format::{DelayedFormat, StrftimeItems},
    DateTime, Datelike, Duration, NaiveDate, Utc,
    Weekday::{Fri, Mon, Sat, Sun, Thu, Tue, Wed},
};
use core::fmt;

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

pub struct Calendar {
    delivery_dates: Vec<DeliveryDate>,
    created: Option<DateTime<Utc>>,
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
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("BEGIN:VCALENDAR\r\n")?;
        f.write_str("VERSION:2.0\r\n")?;
        f.write_str("PRODID:-//Aasan//Aasan Postgang//EN\r\n")?;
        f.write_str("CALSCALE:GREGORIAN\r\n")?;
        f.write_str("METHOD:PUBLISH\r\n")?;

        for value in &self.delivery_dates {
            f.write_str("BEGIN:VEVENT\r\n")?;
            write!(
                f,
                "DTEND;VALUE=DATE:{}\r\n",
                format_naive_date(value.date + Duration::days(1))
            )?;
            write!(
                f,
                "DTSTAMP:{}\r\n",
                format_timestamp(&(self.created.unwrap_or(Utc::now())))
            )?;
            write!(
                f,
                "DTSTART;VALUE=DATE:{}\r\n",
                format_naive_date(value.date)
            )?;
            write!(
                f,
                "SUMMARY:{}: Posten kommer {} {}.\r\n",
                value.postal_code,
                weekday(value.date),
                value.date.day()
            )?;
            f.write_str("TRANSP:TRANSPARENT\r\n")?;
            write!(f, "UID:postgang-{}-{}\r\n", value.postal_code, value.date)?;
            f.write_str("URL:https://www.posten.no/levering-av-post/\r\n")?;
            f.write_str("END:VEVENT\r\n")?;
        }

        f.write_str("END:VCALENDAR\r\n")
    }
}
