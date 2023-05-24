use chrono::NaiveDate;
use core::fmt::{self, Display};
use std::error::Error;

#[derive(Debug, Clone)]
pub struct PostalCode(String);

#[derive(Debug)]
pub struct PostalCodeError;

impl Display for PostalCodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(
            "Invalid postal code format. Postal code must be numeric and consist of 4 digits",
        )
    }
}

impl Error for PostalCodeError {}

impl<'a> TryFrom<&'a str> for PostalCode {
    type Error = PostalCodeError;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        if value.len() != 4 || !value.bytes().all(|c| c.is_ascii_digit()) {
            Err(PostalCodeError)
        } else {
            Ok(Self(value.to_owned()))
        }
    }
}

impl Display for PostalCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

#[derive(Debug)]
pub struct DeliveryDate<'a> {
    pub postal_code: &'a PostalCode,
    pub date: NaiveDate,
}

impl<'a> DeliveryDate<'a> {
    pub fn new(postal_code: &'a PostalCode, date: NaiveDate) -> Self {
        Self { postal_code, date }
    }
}

pub trait DeliveryDateProvider<'a> {
    fn get(&'a self, postal_code: &'a PostalCode) -> Result<Vec<DeliveryDate>, String>;
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

        impl<'a> DeliveryDateProvider<'a> for Endpoint {
            fn get(&'a self, postal_code: &'a PostalCode) -> Result<Vec<DeliveryDate>, String> {
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
                                .map(|date| DeliveryDate::new(postal_code, *date))
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

    impl<'a> From<&'a DeliveryDate<'a>> for Event {
        fn from(value: &DeliveryDate<'a>) -> Self {
            let weekday = match value.date.weekday() {
                Mon => "mandag",
                Tue => "tirsdag",
                Wed => "onsdag",
                Thu => "torsdag",
                Fri => "fredag",
                Sat => "lørdag",
                Sun => "søndag",
            };
            Event::new()
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
                .done()
        }
    }

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
}
