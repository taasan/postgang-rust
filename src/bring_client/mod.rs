//! Client for the [Bring postal code API](https://developer.bring.com/api/postal-code/).
use core::fmt::{self, Display};
use reqwest::header::HeaderValue;
use std::error::Error;
use std::fmt::Debug;

const HEADER_UID: &str = "X-Mybring-API-Uid";
const HEADER_KEY: &str = "X-Mybring-API-Key";
const NORWAY: &str = "no";
const INVALID_NORWEGIAN_POST_CODE: &str =
    "Invalid postal code format for Norway. Postal code must be numeric and consist of 4 digits";

#[derive(Debug, Clone)]
/// Represents a norwegian postal code.
///
/// Postal codes must be numeric and consist of 4 digits
///
/// ```
/// use postgang::bring_client::NorwegianPostalCode;
/// let postal_code = NorwegianPostalCode::try_from("0001").unwrap();
/// assert_eq!(format!("{}", postal_code), "0001");
/// ```
/// ```rust,should_panic
/// use postgang::bring_client::NorwegianPostalCode;
/// NorwegianPostalCode::try_from("10000").unwrap();
/// ```
pub struct NorwegianPostalCode(u16);

#[derive(Debug)]
/// A possible error when converting a PostalCode from a string.
pub struct InvalidPostalCode(&'static str);

impl Display for InvalidPostalCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl Error for InvalidPostalCode {}

impl<'a> TryFrom<&'a str> for NorwegianPostalCode {
    type Error = InvalidPostalCode;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        if value.len() != 4 || !value.bytes().all(|c| c.is_ascii_digit()) {
            Err(InvalidPostalCode(INVALID_NORWEGIAN_POST_CODE))
        } else {
            Ok(Self(value.parse().map_err(|_| {
                InvalidPostalCode(INVALID_NORWEGIAN_POST_CODE)
            })?))
        }
    }
}

impl Display for NorwegianPostalCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{:04}", self.0))
    }
}

#[derive(Clone)]
/// API key to be used by the API client.
pub struct ApiKey(HeaderValue);

impl ApiKey {
    /// Create a new `ApiKey` from `HeaderValue`.
    ///
    /// The header is marked sensitive as to not leak secrets in log output.
    ///
    /// ```
    /// use reqwest::header::{HeaderMap, HeaderValue};
    /// use postgang::bring_client::ApiKey;
    /// let value = ApiKey::new(HeaderValue::from_static("secret value"));
    /// assert_eq!(format!("{:?}", value), "ApiKey(Sensitive)");
    /// ```
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

pub mod mailbox_delivery_dates;
