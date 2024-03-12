
use std::num::ParseIntError;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CommandError {

    #[error("Unsupported protocol version: {}", .version)]
    UnsupportedProtocolVersion {version: String},

    #[error("Invalid Transaction ID: {}", .tr_id)]
    InvalidTrId {tr_id: String, source: ParseIntError},

    #[error("Too many argument for command {}, expected: {} and received {}", .command, .expected, .received)]
    TooManyArguments {command: String, expected: u32, received: u32},

    #[error("Could not parse argument: {} for command : {}", .argument, .command)]
    ArgumentParseError{argument: String, command: String, source: anyhow::Error}
}