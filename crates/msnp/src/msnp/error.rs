
use std::{num::ParseIntError, str::Utf8Error};
use std::string::FromUtf8Error;
use hex::FromHexError;
use strum::ParseError;
use crate::shared::errors::IdentifierError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommandError {

    #[error("Unsupported protocol version: {}", .version)]
    UnsupportedProtocolVersion {version: String},

    #[error("Wrong argument count for command {}, expected: {} and received {}", .command, .expected, .received)]
    WrongArgumentCount {command: String, expected: u32, received: u32},

    #[error("Missing argument {} at index {} for commmand {}", .1, .2, .0)]
    MissingArgument (String, String, usize),

    #[error("Could not parse argument: {} for command : {}", .argument, .command)]
    ArgumentParseError{argument: String, command: String, source: anyhow::Error},

    #[error("Payload command was malformed")]
    MalformedPayloadCommand { source: anyhow::Error },

    #[error("Command was unsupported: {}", .command)]
    UnsupportedCommand { command: String },

    #[error("No command to extract in buffer: {:?}", .buffer)]
    NoCommandToExtract { buffer: Vec<u8>},


    #[error(transparent)]
    UTF8Error(#[from] Utf8Error),
    #[error(transparent)]
    FromUTF8Error(#[from] FromUtf8Error),

    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),

    #[error(transparent)]
    ParseError(#[from] ParseError),

    #[error(transparent)]
    IdentifierError(#[from] IdentifierError),

    #[error(transparent)]
    PayloadError(#[from] PayloadError)
}

#[derive(Error, Debug)]
pub enum PayloadError {
    #[error("Couldn't parse the payload: {}", .payload)]
    StringPayloadParsingError {
        payload: String,
        source: anyhow::Error
    },
    #[error("Couldn't parse the binary payload: {:?}", .payload)]
    BinaryPayloadParsingError {
        payload: Vec<u8>,
        source: anyhow::Error
    },
    #[error(transparent)]
    Utf8Error(#[from] Utf8Error),
    #[error(transparent)]
    FromUtf8Error(#[from] FromUtf8Error),
    #[error("Couldn't parse enum: {:?}", .payload)]
    EnumParsingError {
        payload: String,
        source: anyhow::Error
    },
    #[error("The payload was chunked & not complete")]
    PayloadBytesMissing,
    #[error("The payload did not contain SLP packet")]
    PayloadDoesNotContainsSLP,
    #[error("The payload type is unknown to us & not handled {}", .payload)]
    PayloadNotHandled {payload: String},
    #[error("The payload did not contain a mandatory part {} - payload: {:?}", .name, .payload)]
    MandatoryPartNotFound{ name: String, payload: String},
    #[error("The property {} of payload_type: {} could not be parsed: raw_value: {}", .property_name, .payload_type, .raw_value)]
    PayloadPropertyParseError { property_name: String, raw_value: String, payload_type: String, source: anyhow::Error},
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
    #[error("Payload was missing from command {}", .command)]
    MissingPayload{command: String},
    #[error("Payload was bigger ({}b) than expected {}: {:?}", .overflowing_size, .expected_size, .payload)]
    PayloadSizeExceed {
        expected_size: usize, overflowing_size: usize, payload: Vec<u8>
    },
    #[error(transparent)]
    HexDecodeError(#[from] FromHexError),
    #[error(transparent)]
    AnyError(#[from] anyhow::Error)

}