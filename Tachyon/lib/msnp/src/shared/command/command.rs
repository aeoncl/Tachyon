use std::str::FromStr;

use crate::msnp::{error::CommandError, raw_command_parser::RawCommand};

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

pub fn get_split_part<'b, 'a>(index: usize, split: &'a [&str], command: &'a str, arg_name: &'b str) -> Result<&'a str, CommandError> {
    Ok(split
        .get(index)
        .ok_or(CommandError::MissingArgument { command: command.to_string(), arg_name: arg_name.to_string(), index })?
    )  
}

pub fn parse_tr_id(splitted_command: &Vec<&str>) -> Result<u128, CommandError> {
    let tr_id_as_str = splitted_command.get(1).expect("tr_id to be present");
    u128::from_str(tr_id_as_str).map_err(|e| CommandError::InvalidTrId {
        tr_id: tr_id_as_str.to_string(),
        source: e,
    })
}

pub trait MSNPCommand {
    fn get_operand(&self) -> &str;
}

pub trait ParseRawCommand<T> {
    fn parse_raw_command(command: &RawCommand) -> T;
}
