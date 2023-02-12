

pub enum SwitchboardError {
    MatrixSdkError(matrix_sdk::Error),
    MatrixRoomNotFound,
    MimeError(mime::FromStrError),
    UnknownError
}

impl From<matrix_sdk::Error> for SwitchboardError {
    fn from(err: matrix_sdk::Error) -> SwitchboardError {
        SwitchboardError::MatrixSdkError(err)
    }
}

impl From<mime::FromStrError> for SwitchboardError {
    fn from(err: mime::FromStrError) -> SwitchboardError {
        SwitchboardError::MimeError(err)
    }
}
