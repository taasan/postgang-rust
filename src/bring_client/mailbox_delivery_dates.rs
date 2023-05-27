//! Mailbox delivery dates API.

use super::ApiKey;
use super::NorwegianPostalCode;
use super::NORWAY;
use chrono::{DateTime, NaiveDate, Utc};
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;
use std::fmt::Debug;
use std::path::PathBuf;

#[derive(Debug)]
/// Represents a mailbox delivery date for a specific postal code.
pub struct DeliveryDate<'a> {
    pub(crate) postal_code: &'a NorwegianPostalCode,
    pub(crate) date: NaiveDate,
    pub(crate) created: DateTime<Utc>,
}

impl<'a> DeliveryDate<'a> {
    pub fn new(
        postal_code: &'a NorwegianPostalCode,
        date: NaiveDate,
        created: DateTime<Utc>,
    ) -> Self {
        Self {
            postal_code,
            date,
            created,
        }
    }
}

#[derive(Deserialize, Debug)]
/// Represents JSON structure from the API.
struct ApiResponse {
    delivery_dates: Vec<NaiveDate>,
}

struct ApiResponseWithPostalCode<'a> {
    response: ApiResponse,
    postal_code: &'a NorwegianPostalCode,
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

/// Delivery day provider.        
pub enum DeliveryDays {
    /// Fetches JSON from [Bring API](https://developer.bring.com/api/postal-code/#get-mailbox-delivery-dates-at-postal-code-get).
    // https://api.bring.com/address/api/{country-code}/postal-codes/{postal-code}/mailbox-delivery-dates
    Api(Client),

    /// Reads JSON from a file.
    File(PathBuf),
}

impl DeliveryDays {
    /// Read dates from REST API.
    pub fn api(api_key: ApiKey, api_uid: HeaderValue) -> Self {
        let mut headers = HeaderMap::with_capacity(3);
        headers.insert("accept", HeaderValue::from_str("application/json").unwrap());
        headers.insert(super::HEADER_UID, api_uid);
        headers.insert(super::HEADER_KEY, api_key.0);
        log::debug!("Constructing HTTP client with headers: {:?}", headers);
        let client = Client::builder().default_headers(headers).build().unwrap();
        Self::Api(client)
    }

    /// Read dates from file.
    pub fn file(path: PathBuf) -> Self {
        Self::File(path)
    }

    /// Get a list of delivery dates.
    pub fn get<'a>(
        &'a self,
        postal_code: &'a NorwegianPostalCode,
    ) -> Result<Vec<DeliveryDate>, Box<dyn std::error::Error>> {
        let response: ApiResponse = match self {
            Self::Api(client) => {
                let url = format!(
                    "https://api.bring.com/address/api/{}/postal-codes/{}/mailbox-delivery-dates",
                    NORWAY, postal_code
                );
                log::debug!("Using URL: {url}");
                let resp = client.get(&url).send()?;
                log::debug!("Got response status: {}", resp.status());
                log::trace!("{:?}", resp);
                resp.error_for_status_ref()?;
                resp.json::<ApiResponse>()?
            }
            Self::File(path) => {
                log::debug!("Reading from file: {:?}", path);
                serde_json::from_reader(std::fs::File::open(path)?)?
            }
        };
        log::debug!("Got: {:?}", response);
        let rwpc = ApiResponseWithPostalCode {
            response,
            postal_code,
        };
        Ok(rwpc.into())
    }
}
