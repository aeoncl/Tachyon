use thiserror::Error;
#[derive(Error, Debug)]
pub enum TachyonError {

    #[error(transparent)]
    MatrixConversion(#[from] MatrixConversionError)

}

#[derive(Error, Debug)]
pub enum MatrixConversionError {
    #[error("Could not convert Email to Matrix ID: {}", .email)]
    EmailToMatrixId {email: String, source: anyhow::Error},
    #[error("Could not generate Device Id")]
    DeviceIdGeneration { source: anyhow::Error}

}