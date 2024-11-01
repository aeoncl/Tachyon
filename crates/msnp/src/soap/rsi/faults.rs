use yaserde::ser::to_string;
use yaserde_derive::{YaDeserialize, YaSerialize};
use crate::soap::abch::msnab_faults::FaultErrorCode;
use crate::soap::error::SoapMarshallError;
use crate::soap::traits::xml::ToXml;

#[cfg(test)]
mod tests {
    use yaserde::de::from_str;
    use yaserde::ser::to_string;

    use crate::shared::models::uuid::Uuid;
    use crate::soap::traits::xml::ToXml;
    use super::SoapFaultResponseEnvelope;

    #[test]
    fn test_deserialize() {
        let req = r#"<?xml version="1.0" encoding="utf-8"?>
                            <soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema">
                              <soap:Body>
                                <soap:Fault>
                                  <faultcode xmlns:q0="http://messenger.msn.com/ws/2004/09/oim/">q0:AuthenticationFailed</faultcode>
                                  <faultstring>Exception of type System.Web.Services.Protocols.SoapException was thrown.</faultstring>
                                  <faultactor>https://ows.messenger.msn.com/OimWS/oim.asmx</faultactor>
                                  <detail>
                                    <TweenerChallenge xmlns="http://messenger.msn.com/ws/2004/09/oim/">lc=1033,id=507,tw=120,ru=http://messenger.msn.com,ct=1132075720,kpp=1,kv=7,ver=2.1.6000.1,tpf=ab7bfa98f7683164c11c7dba276daa58</TweenerChallenge>
                                    <LockKeyChallenge xmlns="http://messenger.msn.com/ws/2004/09/oim/">1850937852</LockKeyChallenge>
                                  </detail>
                                </soap:Fault>
                              </soap:Body>
                            </soap:Envelope>"#;

        let deser: SoapFaultResponseEnvelope = from_str(&req).expect("to work");

        // FIXME don't be surprised if this breaks in a newer version of yaserde
        // assert_eq!("http://messenger.msn.com/ws/2004/09/oim/", deser.body.fault.fault_code.xmlns.expect("to be here"));
        assert_eq!("q0:AuthenticationFailed", deser.body.fault.fault_code.value);
        assert_eq!("lc=1033,id=507,tw=120,ru=http://messenger.msn.com,ct=1132075720,kpp=1,kv=7,ver=2.1.6000.1,tpf=ab7bfa98f7683164c11c7dba276daa58", deser.body.fault.detail.as_ref().expect("to be here").tweener_challenge.as_ref().expect("to be here").value);
        assert_eq!("1850937852", deser.body.fault.detail.as_ref().expect("to be here").lock_key_challenge.as_ref().expect("to be here").value);
    }

    #[test]
    fn test_ser() {
        let fault = SoapFaultResponseEnvelope::new_authentication_failed("https://ows.messenger.msn.com/OimWS/oim.asmx", Some("Hey".into()), Some("1234".into()));
        let ser = fault.to_xml().unwrap();
        println!("{}", ser);
    }

}


#[derive(Debug, Default, YaSerialize, YaDeserialize)]
#[yaserde(
rename = "Envelope",
namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance"
namespace = "xsd: http://www.w3.org/2001/XMLSchema"
prefix = "soap"
)]
pub struct SoapFaultResponseEnvelope {
    #[yaserde(rename = "Body", prefix = "soap")]
    body: SoapFaultBody
}

impl ToXml for SoapFaultResponseEnvelope {
    type Error = SoapMarshallError;

    fn to_xml(&self) -> Result<String, Self::Error>  {
        to_string(self).map_err(|e| SoapMarshallError::SerializationError { message: e})
    }
}

impl SoapFaultResponseEnvelope {

    pub fn new_authentication_failed(soap_serv_url: &str, tweener_challenge: Option<String>, lock_key_challenge: Option<String>) -> Self {


        let fault_detail = FaultDetail {
            tweener_challenge: tweener_challenge.map(|t| Challenge{ value: t } ),
            lock_key_challenge: lock_key_challenge.map(|l| Challenge{ value: l}),
        };

        let fault_code = FaultCode {
            xmlns: Some("http://messenger.msn.com/ws/2004/09/oim/".into()),
            value: "q0:AuthenticationFailed".to_string(),
        };

        let soap_fault = SoapFault {
            fault_code,
            fault_string: Some("Exception of type System.Web.Services.Protocols.SoapException was thrown.".into()),
            fault_actor: Some(soap_serv_url.to_owned()),
            detail: Some(fault_detail),
        };

        SoapFaultResponseEnvelope {
            body: SoapFaultBody {
                fault: soap_fault,
            }
        }
    }

    /*
    This error is usually being sent to you when sending an OIM, in the following situations:
        - The user does not exist
        - The user is not on your AL list (you cannot send messages to blocked contacts)
    */
    pub fn new_system_unavailable() -> Self {
        let soap_fault = SoapFault {
            fault_code: FaultCode { xmlns: Some("http://messenger.msn.com/ws/2004/09/oim/".into()), value: "q0:SystemUnavailable".into() },
            fault_string: Some("Exception of type System.Web.Services.Protocols.SoapException was thrown.".into()),
            fault_actor: None,
            detail: None,
        };

        SoapFaultResponseEnvelope {
            body: SoapFaultBody {
                fault: soap_fault,
            }
        }
    }

    pub fn new_schema_validator_error(soap_serv_url: &str) -> Self {
        let soap_fault = SoapFault {
            fault_code: FaultCode { xmlns: None, value: "soap:Client".into() },
            fault_string: Some("Schema validation error".into()),
            fault_actor: Some(soap_serv_url.into()),
            detail: None,
        };

        SoapFaultResponseEnvelope {
            body: SoapFaultBody {
                fault: soap_fault,
            }
        }

    }

    pub fn new_generic(fault_string: String) -> Self {
        let soap_fault = SoapFault{
            fault_code: FaultCode { xmlns: None, value: "soap:Client".into() },
            fault_string: Some(fault_string),
            fault_actor: None,
            detail: None,
        };

        SoapFaultResponseEnvelope {
            body: SoapFaultBody {
                fault: soap_fault,
            }
        }
    }

    pub fn new_send_throttle_limit_exceed() -> Self {
        let soap_fault = SoapFault {
            fault_code: FaultCode { xmlns: Some("http://messenger.msn.com/ws/2004/09/oim/".into()), value: "q0:SenderThrottleLimitExceeded".into() },
            fault_string: Some("Exception of type 'System.Web.Services.Protocols.SoapException' was thrown.".into()),
            fault_actor: None,
            detail: None,
        };

        SoapFaultResponseEnvelope {
            body: SoapFaultBody {
                fault: soap_fault,
            }
        }
    }

    pub fn new_unknown_soap_action(soap_action: String) -> Self {

        let soap_fault = SoapFault {
            fault_code: FaultCode{ xmlns: None, value: "SOAP-ENV:Server".to_string() },
            fault_string: Some(format!("Function '{}' doesn't exist", soap_action)),
            fault_actor: None,
            detail: None,
        };

     SoapFaultResponseEnvelope {
            body: SoapFaultBody {
                fault: soap_fault,
            }
        }
    }

}



#[derive(Debug, Default, YaSerialize, YaDeserialize)]
#[yaserde(
rename = "Body",
namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
prefix = "soap"
)]
pub struct SoapFaultBody {
    #[yaserde(rename = "Fault", prefix="soap")]
    fault: SoapFault
}


#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
rename = "faultcode",
namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
prefix = "soap",
)]
pub struct FaultCode {

    #[yaserde(attribute, rename="xmlns:q0")]
    pub xmlns: Option<String>,

    #[yaserde(text)]
    pub value: String

}

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
rename = "Fault",
namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
prefix = "soap",
)]
pub struct SoapFault {
    #[yaserde(rename = "faultcode", default)]
    pub fault_code: FaultCode,
    #[yaserde(rename = "faultstring", default)]
    pub fault_string: Option<String>,
    #[yaserde(rename = "faultactor", default)]
    pub fault_actor: Option<String>,
    #[yaserde(rename = "detail", default)]
    pub detail: Option<FaultDetail>
}

#[derive(Debug, YaSerialize, YaDeserialize, Clone)]
#[yaserde(rename = "detail")]
pub struct FaultDetail {
    #[yaserde(rename = "TweenerChallenge", default)]
    pub tweener_challenge: Option<Challenge>,
    #[yaserde(rename = "LockKeyChallenge", default)]
    pub lock_key_challenge: Option<Challenge>
}

#[derive(Debug, YaSerialize, YaDeserialize, Clone)]
#[yaserde(rename = "TweenerChallenge",
namespace="q0: http://messenger.msn.com/ws/2004/09/oim/",
default_namespace="q0"
)]
pub struct Challenge {
    #[yaserde(text)]
    pub value: String
}


