use crate::matrix::application::login::{MatrixLoginService, MatrixLoginServiceImpl};

pub const MATRIX_LOGIN_SERVICE: Box<dyn MatrixLoginService> = Box::new(MatrixLoginServiceImpl::new());