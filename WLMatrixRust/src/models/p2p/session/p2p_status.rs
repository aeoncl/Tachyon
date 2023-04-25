use num_derive::FromPrimitive;

#[derive(Clone, Copy, Debug, FromPrimitive)]
pub enum P2PSessionStatus {
    WAITING,
    ONGOING,
    CANCELLED,
    DONE
}