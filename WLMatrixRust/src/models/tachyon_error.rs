use std::{num::ParseIntError, str::Utf8Error};

#[derive(Debug)]

pub enum TachyonError {

    PayloadDeserializeError,
	PayloadDoesNotContainsSLP,
	PayloadNotComplete

}

impl From<Utf8Error> for TachyonError {
    fn from(err: Utf8Error) -> TachyonError {
        TachyonError::PayloadDeserializeError
    }
}

impl From<ParseIntError> for TachyonError {
	fn from(err: ParseIntError) -> TachyonError {
        TachyonError::PayloadDeserializeError
    }
}