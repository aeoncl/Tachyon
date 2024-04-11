use yaserde_derive::{YaDeserialize, YaSerialize};


#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
pub struct SpaceHeaders {

    #[yaserde(rename = "AuthTokenHeader", default)]
    pub auth_token: Option<AuthTokenHeader>

}


#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
rename = "AuthTokenHeader",
namespace = "tns: http://www.msn.com/webservices/spaces/v1/",
namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
prefix = "tns",
)]
pub struct AuthTokenHeader {
    #[yaserde(rename = "Token", default)]
    pub token: String,
}

