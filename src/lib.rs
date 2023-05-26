use chrono::{DateTime, NaiveDate, Utc};
use core::fmt::{self, Display};

use std::error::Error;
#[derive(Debug, Clone)]
pub struct PostalCode(u16);

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
            Ok(Self(value.parse().map_err(|_| PostalCodeError)?))
        }
    }
}

impl Display for PostalCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:04}", self.0))
    }
}

#[derive(Debug)]
pub struct DeliveryDate<'a> {
    pub postal_code: &'a PostalCode,
    pub date: NaiveDate,
    pub created: DateTime<Utc>,
}

impl<'a> DeliveryDate<'a> {
    pub fn new(postal_code: &'a PostalCode, date: NaiveDate, created: DateTime<Utc>) -> Self {
        Self {
            postal_code,
            date,
            created,
        }
    }
}

pub mod bring_client {
    const HEADER_UID: &str = "X-Mybring-API-Uid";
    const HEADER_KEY: &str = "X-Mybring-API-Key";

    pub mod mailbox_delivery_dates {
        use crate::{DeliveryDate, PostalCode};
        use chrono::NaiveDate;
        use reqwest::blocking::Client;
        use reqwest::header::{HeaderMap, HeaderValue};
        use serde::Deserialize;
        use std::fmt::Debug;
        use std::path::PathBuf;

        #[derive(Clone)]
        pub struct ApiKey(HeaderValue);

        impl ApiKey {
            pub fn new(value: HeaderValue) -> Self {
                if !value.is_sensitive() {
                    let mut value = value;
                    value.set_sensitive(true);
                    Self(value)
                } else {
                    Self(value)
                }
            }
        }

        impl Debug for ApiKey {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_tuple("ApiKey").field(&self.0).finish()
            }
        }

        #[derive(Deserialize, Debug)]
        struct ApiResponse {
            pub delivery_dates: Vec<NaiveDate>,
        }

        struct ApiResponseWithPostalCode<'a> {
            response: ApiResponse,
            postal_code: &'a PostalCode,
        }

        impl<'a> From<ApiResponseWithPostalCode<'a>> for Vec<DeliveryDate<'a>> {
            fn from(value: ApiResponseWithPostalCode<'a>) -> Self {
                let now = chrono::Utc::now();
                value
                    .response
                    .delivery_dates
                    .iter()
                    .map(|date| DeliveryDate::new(value.postal_code, *date, now))
                    .collect()
            }
        }

        // https://developer.bring.com/api/postal-code/#get-mailbox-delivery-dates-at-postal-code-get
        // https://api.bring.com/address/api/{country-code}/postal-codes/{postal-code}/mailbox-delivery-dates
        pub enum Endpoint {
            Api(Client),
            File(PathBuf),
        }

        impl Endpoint {
            pub fn file(path: PathBuf) -> Self {
                Self::File(path)
            }

            pub fn api(api_key: ApiKey, api_uid: HeaderValue) -> Self {
                let mut headers = HeaderMap::with_capacity(3);
                headers.insert("accept", HeaderValue::from_str("application/json").unwrap());
                headers.insert(super::HEADER_UID, api_uid);
                headers.insert(super::HEADER_KEY, api_key.0);
                log::info!("Constructing HTTP client with headers: {:?}", headers);
                let client = Client::builder().default_headers(headers).build().unwrap();
                Self::Api(client)
            }

            pub fn get<'a>(
                &'a self,
                postal_code: &'a PostalCode,
            ) -> Result<Vec<DeliveryDate>, Box<dyn std::error::Error>> {
                let response: ApiResponse = match self {
                    Self::Api(client) => {
                        let url = format!(
                            "https://api.bring.com/address/api/{}/postal-codes/{}/mailbox-delivery-dates",
                            "no", postal_code
                        );
                        log::info!("Using URL: {url}");
                        let resp = client.get(&url).send()?;
                        log::info!("{:?}", resp.headers());
                        log::info!("Got response status: {}", resp.status());
                        resp.error_for_status_ref()?;
                        resp.json::<ApiResponse>()?
                    }
                    Self::File(path) => {
                        log::info!("Reading from file: {:?}", path);
                        serde_json::from_reader(std::fs::File::open(path)?)?
                    }
                };
                log::info!("Got: {:?}", response);
                let rwpc = ApiResponseWithPostalCode {
                    response,
                    postal_code,
                };
                Ok(rwpc.into())
            }
        }

        #[cfg(test)]
        mod tests {
            use crate::bring_client::mailbox_delivery_dates::ApiKey;
            use reqwest::header::HeaderValue;

            #[test]
            fn api_key_header_becomes_sensitive() {
                let value = HeaderValue::from_static("secret value");
                assert!(!value.is_sensitive());
                let key = ApiKey::new(value);
                assert!(key.0.is_sensitive())
            }

            #[test]
            fn api_key_header_stays_sensitive() {
                let mut value = HeaderValue::from_static("secret value");
                value.set_sensitive(true);
                let key = ApiKey::new(value);
                assert!(key.0.is_sensitive())
            }

            #[test]
            fn api_key_header_debug_print() {
                let value = HeaderValue::from_static("secret value");
                let value = ApiKey::new(value);
                let s = format!("{:?}", value);
                assert_eq!(s, "ApiKey(Sensitive)".to_string());
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

    #[cfg(test)]
    mod tests {
        use crate::{DeliveryDate, PostalCode};
        use chrono::NaiveDate;
        use icalendar::{Component, Event};

        #[test]
        fn test_event_from_delivery_date() {
            let code = &PostalCode::try_from("7530").unwrap();
            let now = chrono::Utc::now();
            let date = &DeliveryDate::new(code, NaiveDate::default(), now);
            let now = now.format("%Y%m%dT%H%M%SZ").to_string();
            let event: Event = date.into();
            let expected = format!("BEGIN:VEVENT\r\nDTSTAMP:{now}\r\nDTEND;VALUE=DATE:19700102\r\nDTSTART;VALUE=DATE:19700101\r\nSUMMARY:7530: Posten kommer torsdag 1.\r\nTRANSP:TRANSPARENT\r\nUID:postgang-7530-1970-01-01\r\nURL:https://www.posten.no/levering-av-post/\r\nEND:VEVENT\r\n");
            assert_eq!(event.to_string(), expected);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::PostalCode;

    #[test]
    fn postal_code_7530_display() {
        let code = PostalCode::try_from("7530").unwrap();
        assert_eq!(code.0, 7530);
        let code = format!("{code}");
        assert_eq!(&code, "7530")
    }

    #[test]
    fn postal_code_0001_display() {
        let code = PostalCode::try_from("0001").unwrap();
        assert_eq!(code.0, 1);
        let code = format!("{code}");
        assert_eq!(&code, "0001")
    }
}
