use core::fmt;
use std::{error::Error, fmt::Display};

use chrono::NaiveDate;

#[derive(Debug, Clone)]
pub struct PostalCode(String);

#[derive(Debug, Clone)]
pub struct PostalCodeError;

impl Display for PostalCodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(
            "Invalid postal code format. Postal code must be numeric and consist of 4 digits",
        )
    }
}

impl Error for PostalCodeError {}

impl TryFrom<&str> for PostalCode {
    type Error = PostalCodeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.len() != 4 || !value.bytes().all(|c| c.is_ascii_digit()) {
            Err(PostalCodeError)
        } else {
            Ok(Self(value.to_owned()))
        }
    }
}

impl fmt::Display for PostalCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

#[derive(Debug, Clone)]
pub struct DeliveryDate {
    pub postal_code: PostalCode,
    pub date: NaiveDate,
}

impl DeliveryDate {
    pub fn new(postal_code: PostalCode, date: NaiveDate) -> Self {
        Self { postal_code, date }
    }
}

pub trait DeliveryDateProvider {
    fn get(&self, postal_code: &PostalCode) -> core::result::Result<Vec<DeliveryDate>, String>;
}

pub mod bring_client {
    pub mod mailbox_delivery_dates {
        use crate::{DeliveryDate, DeliveryDateProvider, PostalCode};
        use chrono::NaiveDate;
        use reqwest::blocking::Client;
        use reqwest::header::{HeaderMap, HeaderValue};

        use serde::Deserialize;

        #[derive(Deserialize, Debug)]
        struct ApiResponse {
            pub delivery_dates: Vec<NaiveDate>,
        }

        // https://developer.bring.com/api/postal-code/#get-mailbox-delivery-dates-at-postal-code-get
        // https://api.bring.com/address/api/{country-code}/postal-codes/{postal-code}/mailbox-delivery-dates
        pub struct Endpoint {
            client: Client,
        }

        impl Endpoint {
            pub fn new(api_key: &str, api_uid: &str) -> Self {
                let mut headers = HeaderMap::new();
                headers.insert("X-Mybring-API-Uid", HeaderValue::from_str(api_uid).unwrap());
                headers.insert("X-Mybring-API-Key", HeaderValue::from_str(api_key).unwrap());
                let client = Client::builder().default_headers(headers).build().unwrap();
                Self { client }
            }
        }

        impl DeliveryDateProvider for Endpoint {
            fn get(&self, postal_code: &PostalCode) -> Result<Vec<DeliveryDate>, String> {
                let url = format!(
                    "https://api.bring.com/address/api/{}/postal-codes/{}/mailbox-delivery-dates",
                    "no", postal_code
                );
                let resp = self.client.get(&url).send().map_err(|x| x.to_string())?;
                if resp.status().is_client_error() {
                    Err(format!(
                        "URL: {url}, HTTP Status: {}, body: {}",
                        resp.status(),
                        resp.text().map_err(|x| x.to_string())?
                    ))
                } else {
                    resp.error_for_status_ref().map_err(|x| x.to_string())?;
                    resp.json::<ApiResponse>()
                        .map(|response| {
                            response
                                .delivery_dates
                                .iter()
                                .map(|date| DeliveryDate::new(postal_code.clone(), *date))
                                .collect()
                        })
                        .map_err(|x| x.to_string())
                }
            }
        }
    }
}

pub mod calendar {
    use crate::DeliveryDate;
    use chrono::{Datelike, Duration, Weekday::*};
    use icalendar::{Calendar, Component, Event, EventLike, Property};

    pub fn delivery_date_to_event(delivery_date: &DeliveryDate) -> Event {
        let date = delivery_date.date;
        let weekday = match date.weekday() {
            Mon => "mandag",
            Tue => "tirsdag",
            Wed => "onsdag",
            Thu => "torsdag",
            Fri => "fredag",
            Sat => "lørdag",
            Sun => "søndag",
        };
        Event::new()
            .uid(
                format!(
                    "postgang-{}-{}",
                    delivery_date.postal_code, delivery_date.date
                )
                .as_str(),
            )
            .url("https://www.posten.no/levering-av-post/")
            .summary(
                format!(
                    "{}: Posten kommer {} {}.",
                    delivery_date.postal_code,
                    weekday,
                    date.day()
                )
                .as_str(),
            )
            .starts(date)
            .ends(date + Duration::days(1))
            .append_property(Property::new("TRANSP", "TRANSPARENT").done())
            .done()
    }

    pub fn to_calendar(delivery_dates: Vec<DeliveryDate>) -> String {
        let mut cal = Calendar::empty();
        cal.append_property(("VERSION", "2.0"));
        cal.append_property(("PRODID", "-//Aasan//Aasan Postgang//EN"));
        cal.append_property(("CALSCALE", "GREGORIAN"));
        cal.append_property(("METHOD", "PUBLISH"));
        for date in delivery_dates {
            cal.push(delivery_date_to_event(&date));
        }
        cal.to_string()
    }
}
