
pub enum TachyonError {
    InvalidEmail(String)
}

pub type TachyonResult<T> = Result<T, TachyonError>;