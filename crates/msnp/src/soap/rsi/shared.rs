use yaserde_derive::{YaDeserialize, YaSerialize};

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
rename = "AuthenticationFailed",
namespace = "nsi2: http://www.hotmail.msn.com/ws/2004/09/oim/rsi",
namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
prefix = "nsi2",
)]
pub struct AuthenticationFailed {
    #[yaserde(rename = "TweenerChallenge", default)]
    pub tweener_challenge: String,
}

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
rename = "Fault",
namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
prefix = "soapenv",
)]
pub struct SoapFault {
    #[yaserde(rename = "faultcode", default)]
    pub fault_code: Option<String>,
    #[yaserde(rename = "faultstring", default)]
    pub fault_string: Option<String>,
}

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
rename = "Fault",
namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
prefix = "soapenv",
)]
pub struct SoapAuthenticationFailedException {
    #[yaserde(rename = "faultcode", default)]
    pub fault_code: Option<String>,
    #[yaserde(rename = "faultstring", default)]
    pub fault_string: Option<String>,
    #[yaserde(rename = "AuthenticationFailedMessage", default)]
    pub detail: Option<AuthenticationFailed>,
}