//! Create iCalendar file for norwegian mailbox delivery dates.
use std::io;
use std::path::PathBuf;

pub mod bring_client;
pub mod calendar;

#[inline]
#[must_use]
pub fn io_error_to_string(err: &io::Error, path: &PathBuf) -> String {
    format!("{err}: {path:?}")
}
