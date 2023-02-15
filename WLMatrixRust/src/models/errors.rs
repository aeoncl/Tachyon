use std::{str::Utf8Error, num::ParseIntError};

#[derive(Debug)]

pub enum Errors {

    PayloadDeserializeError,
	PayloadDoesNotContainsSLP,
	PayloadNotComplete

}

impl From<Utf8Error> for Errors {
    fn from(err: Utf8Error) -> Errors {
        Errors::PayloadDeserializeError
    }
}

impl From<ParseIntError> for Errors {
	fn from(err: ParseIntError) -> Errors {
        Errors::PayloadDeserializeError
    }
}