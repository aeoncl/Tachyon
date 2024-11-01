use yaserde_derive::{YaDeserialize, YaSerialize};

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