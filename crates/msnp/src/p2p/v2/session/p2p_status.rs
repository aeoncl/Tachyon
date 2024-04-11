use num_derive::FromPrimitive;

#[derive(Clone, Copy, Debug, FromPrimitive, PartialEq)]
pub enum P2PSessionStatus {
    NONE,
    WAITING,
    ONGOING,
    CANCELLED,
    DONE
}