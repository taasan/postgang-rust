//! iCalendar generator
use super::bring_client::mailbox_delivery_dates::DeliveryDate;
use chrono::{
    format::{DelayedFormat, StrftimeItems},
    DateTime, Datelike, Duration, NaiveDate, Utc,
    Weekday::{Fri, Mon, Sat, Sun, Thu, Tue, Wed},
};

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

#[must_use]
/// Dump delivery dates as an iCalendar string.
///
/// ```
/// use std::str::FromStr;
/// use chrono::{DateTime, FixedOffset, NaiveDate, Utc};
/// use postgang::bring_client::mailbox_delivery_dates::DeliveryDate;
/// use postgang::bring_client::NorwegianPostalCode;
/// use postgang::calendar::to_calendar_string;
///
/// let postal_code = &NorwegianPostalCode::try_from("7800").unwrap();
/// let date = NaiveDate::from_ymd_opt(1970, 8, 13).unwrap();
/// let created = DateTime::<FixedOffset>::parse_from_rfc3339("1970-08-13T00:00:00Z").unwrap().into();
/// let delivery_dates = vec![DeliveryDate::new(postal_code, date)];
/// let ical_str = to_calendar_string(delivery_dates, Some(created));
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
pub fn to_calendar_string(
    delivery_dates: Vec<DeliveryDate>,
    created: Option<DateTime<Utc>>,
) -> String {
    let cap = 7 + 9 * delivery_dates.len();
    let mut buf: Vec<String> = Vec::with_capacity(cap);
    buf.push("BEGIN:VCALENDAR".to_owned());
    buf.push("VERSION:2.0".to_owned());
    buf.push("PRODID:-//Aasan//Aasan Postgang//EN".to_owned());
    buf.push("CALSCALE:GREGORIAN".to_owned());
    buf.push("METHOD:PUBLISH".to_owned());
    for value in delivery_dates {
        buf.push("BEGIN:VEVENT".to_owned());
        buf.push(format!(
            "DTEND;VALUE=DATE:{}",
            format_naive_date(value.date + Duration::days(1))
        ));
        buf.push(format!(
            "DTSTAMP:{}",
            format_timestamp(&(created.unwrap_or(Utc::now())))
        ));
        buf.push(format!(
            "DTSTART;VALUE=DATE:{}",
            format_naive_date(value.date)
        ));
        buf.push(format!(
            "SUMMARY:{}: Posten kommer {} {}.",
            value.postal_code,
            weekday(value.date),
            value.date.day()
        ));
        buf.push("TRANSP:TRANSPARENT".to_owned());
        buf.push(format!("UID:postgang-{}-{}", value.postal_code, value.date));
        buf.push("URL:https://www.posten.no/levering-av-post/".to_owned());
        buf.push("END:VEVENT".to_owned());
    }
    buf.push("END:VCALENDAR".to_owned());
    buf.push(String::new());
    debug_assert!(
        buf.iter().all(|line| line.len() <= 75),
        "Some lines ecceed 75 bytes, implement line folding?"
    );
    debug_assert_eq!(cap, buf.len(), "String buffer initial capacity is wrong");
    buf.join("\r\n")
}
