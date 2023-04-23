#[derive(Clone, Copy, Debug)]
pub enum P2PSessionStatus {
    WAITING,
    ONGOING,
    CANCELLED
}