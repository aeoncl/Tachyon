
use std::num::ParseIntError;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommandError {

    #[error("Unsupported protocol version: {}", .version)]
    UnsupportedProtocolVersion {version: String},

    #[error("Invalid Transaction ID: {}", .tr_id)]
    InvalidTrId {tr_id: String, source: ParseIntError},

    #[error("Wrong argument count for command {}, expected: {} and received {}", .command, .expected, .received)]
    WrongArgumentCount {command: String, expected: u32, received: u32},

    #[error("Missing argument {} at index {} for commmand {}", .arg_name, .index, .command)]
    MissingArgument {command: String, arg_name: String, index: usize},

    #[error("Could not parse argument: {} for command : {}", .argument, .command)]
    ArgumentParseError{argument: String, command: String, source: anyhow::Error},

    #[error("Payload command was malformed")]
    MalformedPayloadCommand { source: anyhow::Error },

    #[error("Command was unsupported: {}", .command)]
    UnsupportedCommand { command: String },

    #[error(transparent)]
    PayloadError(#[from] PayloadError)
}

#[derive(Error, Debug)]
pub enum PayloadError {
    #[error("Couldn't parse the payload: {}", .payload)]
    StringPayloadParsingError {
        payload: String,
        sauce: anyhow::Error

    },
    #[error("Couldn't parse the binary payload: {:?}", .payload)]
    BinaryPayloadParsingError {
        payload: Vec<u8>,
        sauce: anyhow::Error
    },
    #[error("Couldn't parse enum: {:?}", .payload)]
    EnumParsingError {
        payload: String,
        sauce: anyhow::Error
    },
    #[error("The payload was chunked & not complete")]
    PayloadBytesMissing,
    #[error("The payload did not contain SLP packet")]
    PayloadDoesNotContainsSLP,
    #[error("The payload type is unknown to us & not handled {}", .payload)]
    PayloadNotHandled {payload: String},
    #[error("The payload did not contain a mandatory part {} - payload: {:?}", .name, .payload)]
    MandatoryPartNotFound{ name: String, payload: String},
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
    #[error("Payload was missing from command {}", .command)]
    MissingPayload{command: String},
    #[error(transparent)]
    AnyError(#[from] anyhow::Error)

}