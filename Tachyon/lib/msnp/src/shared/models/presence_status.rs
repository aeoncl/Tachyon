use strum_macros::{Display, EnumString};
use yaserde_derive::{YaDeserialize, YaSerialize};


#[derive(Debug, Clone, Display, EnumString, YaSerialize, YaDeserialize, PartialEq, Eq)]
pub enum PresenceStatus {

    /* Online */
    NLN,
    /* Busy */
    BSY,
    /* Away */
    AWY,
    /* Hidden */
    HDN,
    /* Be Right Back */
    BRB,
    /* Idle */
    IDL,
    /* Phone */
    PHN,
    /* Lunch */
    LUN,
    /* Disconnected */
    FLN
}

impl Default for PresenceStatus {

    fn default() -> Self {
        return PresenceStatus::FLN;
    }
}