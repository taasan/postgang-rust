//! iCalendar generator
use super::bring_client::mailbox_delivery_dates::DeliveryDate;
use chrono::{Datelike, Duration, Weekday::*};
use icalendar::{Calendar, Component, Event, EventLike, Property};

impl<'a> From<&'a DeliveryDate<'a>> for Event {
    fn from(value: &DeliveryDate<'a>) -> Self {
        log::trace!("Converting {:?} to Event", value);
        let weekday = match value.date.weekday() {
            Mon => "mandag",
            Tue => "tirsdag",
            Wed => "onsdag",
            Thu => "torsdag",
            Fri => "fredag",
            Sat => "lørdag",
            Sun => "søndag",
        };
        let event = Event::new()
            .timestamp(value.created)
            .uid(format!("postgang-{}-{}", value.postal_code, value.date).as_str())
            .url("https://www.posten.no/levering-av-post/")
            .summary(
                format!(
                    "{}: Posten kommer {} {}.",
                    value.postal_code,
                    weekday,
                    value.date.day()
                )
                .as_str(),
            )
            .starts(value.date)
            .ends(value.date + Duration::days(1))
            .append_property(Property::new("TRANSP", "TRANSPARENT").done())
            .done();
        log::trace!("{:?}", event);
        event
    }
}

/// Dump delivery dates as an iCalendar string.
///
/// ```
/// use std::str::FromStr;
/// use chrono::{DateTime, FixedOffset, NaiveDate, Utc};
/// use postgang::bring_client::mailbox_delivery_dates::DeliveryDate;
/// use postgang::bring_client::NorwegianPostalCode;
/// use postgang::calendar::to_calendar;
///
/// let postal_code = &NorwegianPostalCode::try_from("7800").unwrap();
/// let date = NaiveDate::from_ymd_opt(1970, 8, 13).unwrap();
/// let created = DateTime::<FixedOffset>::parse_from_rfc3339("1970-08-13T00:00:00Z").unwrap().into();
/// let delivery_dates = vec![DeliveryDate::new(postal_code, date, created)];
/// let ical_str = to_calendar(delivery_dates);
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
pub fn to_calendar(delivery_dates: Vec<DeliveryDate>) -> String {
    let mut cal = Calendar::empty();
    cal.append_property(("VERSION", "2.0"));
    cal.append_property(("PRODID", "-//Aasan//Aasan Postgang//EN"));
    cal.append_property(("CALSCALE", "GREGORIAN"));
    cal.append_property(("METHOD", "PUBLISH"));
    for date in delivery_dates {
        cal.push::<Event>((&date).into());
    }
    cal.to_string()
}
