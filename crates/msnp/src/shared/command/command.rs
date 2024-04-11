use std::str::FromStr;

use crate::msnp::error::CommandError;

pub fn split_raw_command(command: &str, argument_count: usize) -> Result<Vec<&str>, CommandError> {
    let split = split_raw_command_no_arg(command);
    if split.len() != argument_count {
        return Err(CommandError::WrongArgumentCount {
            command: command.to_owned(),
            expected: argument_count as u32,
            received: split.len() as u32,
        });
    }

    Ok(split)
}

pub fn split_raw_command_no_arg(command: &str) -> Vec<&str> {
     command.trim_end().split_whitespace().collect::<Vec<&str>>()
}
