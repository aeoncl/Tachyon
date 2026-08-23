pub struct LoginHints {}

pub enum CredentialType {
    Email(String),
}

pub struct LoginStart {
    endpoint: String,
}
