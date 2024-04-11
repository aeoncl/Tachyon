use strum_macros::{Display, EnumString};

#[derive(Debug, EnumString, Display, PartialEq)]
pub enum MsnpVersion {
    MSNP17,
    MSNP18,
    MSNP21
}