use strum_macros::EnumString;

#[derive(Debug, EnumString)]
pub enum MsnpVersion {
    MSNP17,
    MSNP18,
    MSNP21
}