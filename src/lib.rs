//! Create iCalendar file for norwegian mailbox delivery dates.
#![deny(clippy::std_instead_of_core)]
#![deny(clippy::std_instead_of_alloc)]
#![deny(clippy::alloc_instead_of_core)]
#![deny(clippy::complexity)]
#![deny(clippy::pedantic)]
#![deny(unused_qualifications)]

use std::io;
use std::path::PathBuf;

pub mod bring_client;
pub mod calendar;

#[inline]
#[must_use]
pub fn io_error_to_string(err: &io::Error, path: &PathBuf) -> String {
    format!("{err}: {path:?}")
}
