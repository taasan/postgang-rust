use core::fmt;

use chrono::NaiveDate;

#[derive(Debug, Clone)]
pub struct PostalCode(String);

impl From<String> for PostalCode {
    fn from(value: String) -> Self {
        Self(value)
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
    fn get(&self, postal_code: PostalCode) -> core::result::Result<Vec<DeliveryDate>, String>;
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
            pub fn new(api_key: String, api_uid: String) -> Self {
                let mut headers = HeaderMap::new();
                headers.insert(
                    "X-Mybring-API-Uid",
                    HeaderValue::from_str(api_uid.as_str()).unwrap(),
                );
                headers.insert(
                    "X-Mybring-API-Key",
                    HeaderValue::from_str(api_key.as_str()).unwrap(),
                );
                let client = Client::builder().default_headers(headers).build().unwrap();
                Self { client }
            }
        }

        impl DeliveryDateProvider for Endpoint {
            fn get(&self, postal_code: PostalCode) -> Result<Vec<DeliveryDate>, String> {
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
    use chrono::{Datelike, Duration};
    use icalendar::{Calendar, Component, Event, EventLike, Property};

    pub fn delivery_date_to_event(delivery_date: DeliveryDate) -> Event {
        let date = delivery_date.date;
        let weekday = match date.weekday() {
            chrono::Weekday::Mon => "mandag",
            chrono::Weekday::Tue => "tirsdag",
            chrono::Weekday::Wed => "onsdag",
            chrono::Weekday::Thu => "torsdag",
            chrono::Weekday::Fri => "fredag",
            chrono::Weekday::Sat => "lørdag",
            chrono::Weekday::Sun => "søndag",
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
            // .description("here I have something really important to do")
            .starts(date)
            .ends(date + Duration::days(1))
            .append_property(Property::new("TRANSP", "TRANSPARENT").done())
            .done()
    }

    pub fn to_calendar(delivery_dates: Vec<DeliveryDate>) -> String {
        let mut my_calendar = Calendar::empty();
        my_calendar.append_property(("VERSION", "2.0"));
        my_calendar.append_property(("PRODID", "-//Aasan//Aasan Postgang//EN"));
        my_calendar.append_property(("CALSCALE", "GREGORIAN"));
        my_calendar.append_property(("METHOD", "PUBLISH"));
        for date in delivery_dates {
            my_calendar.push(delivery_date_to_event(date));
        }
        my_calendar.to_string()
    }
}
