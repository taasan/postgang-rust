//! Mailbox delivery dates API.

use core::fmt::Debug;
use std::path::PathBuf;

use chrono::NaiveDate;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::{
    bring_client::{ApiKey, ApiUid, NorwegianPostalCode, NORWAY},
    io_error_to_string,
};

#[derive(Debug, Clone, Copy)]
/// Represents a mailbox delivery date for a specific postal code.
pub struct DeliveryDate {
    pub postal_code: NorwegianPostalCode,
    pub date: NaiveDate,
}

impl DeliveryDate {
    #[must_use]
    pub const fn new(postal_code: NorwegianPostalCode, date: NaiveDate) -> Self {
        Self { postal_code, date }
    }
}

#[derive(Serialize, Deserialize, Debug)]
/// Represents JSON structure from the API.
pub struct ApiResponse {
    pub delivery_dates: Vec<NaiveDate>,
}

/// Delivery day provider.
pub enum DeliveryDays {
    /// Fetches JSON from [Bring API](https://developer.bring.com/api/postal-code/#get-mailbox-delivery-dates-at-postal-code-get).
    // https://api.bring.com/address/api/{country-code}/postal-codes/{postal-code}/mailbox-delivery-dates
    Api(Client),

    /// Reads JSON from a file.
    File(Option<PathBuf>),
}

impl DeliveryDays {
    /// Read dates from REST API.
    #[allow(clippy::missing_panics_doc)]
    pub fn api(api_key: ApiKey, api_uid: ApiUid) -> Self {
        let mut headers = HeaderMap::with_capacity(3);
        headers.insert("accept", HeaderValue::from_str("application/json").unwrap());
        headers.insert(super::HEADER_UID, api_uid.0);
        headers.insert(super::HEADER_KEY, api_key.0);
        log::debug!("Constructing HTTP client with headers: {:?}", headers);
        let client = Client::builder().default_headers(headers).build().unwrap();
        Self::Api(client)
    }

    #[must_use]
    /// Read dates from file.
    pub const fn file(path: Option<PathBuf>) -> Self {
        Self::File(path)
    }

    /// Get a list of delivery dates.
    #[allow(clippy::missing_errors_doc)]
    pub async fn get<T: DeserializeOwned>(
        &self,
        postal_code: NorwegianPostalCode,
    ) -> Result<T, Box<dyn core::error::Error>> {
        let response: T = match self {
            Self::Api(client) => {
                let url = format!(
                    "https://api.bring.com/address/api/{NORWAY}/postal-codes/{postal_code}/mailbox-delivery-dates"
                );
                log::debug!("Using URL: {url}");
                let resp = client.get(&url).send().await?;
                log::debug!("Got response status: {}", resp.status());
                log::trace!("{:?}", resp);
                resp.error_for_status_ref()?;
                resp.json().await?
            }
            Self::File(Some(path)) => {
                log::debug!("Reading from file: {:?}", path);
                serde_json::from_reader(
                    std::fs::File::open(path).map_err(|err| io_error_to_string(&err, path))?,
                )?
            }
            Self::File(None) => {
                log::debug!("Reading from stdin");
                serde_json::from_reader(std::io::stdin())?
            }
        };
        Ok(response)
    }
}
