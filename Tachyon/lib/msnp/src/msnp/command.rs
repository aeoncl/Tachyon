use super::{error::CommandError, msnp18::{self}};
use std::str::FromStr;

pub fn split_raw_command(command: &str, argument_count: usize) -> Result<Vec<&str>, CommandError> {

    let split = command.split_whitespace().collect::<Vec<&str>>();
    if split.len() != argument_count {
        return Err(CommandError::TooManyArguments {command: command.to_owned(), expected: argument_count as u32, received: split.len() as u32 });
    }

    Ok(split)
}

pub fn parse_tr_id(splitted_command: &Vec<&str>,) -> Result<u128, CommandError> {

    let tr_id_as_str = splitted_command.get(1).expect("tr_id to be present");
    u128::from_str(tr_id_as_str).map_err(|e| CommandError::InvalidTrId{ tr_id: tr_id_as_str.to_string(), source: e } )
}

pub trait MSNPCommand {
    fn get_operand(&self) -> &str;
}
