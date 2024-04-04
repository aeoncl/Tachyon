//! THIS IS A GENERATED FILE!
//! Take care when hand editing. Changes will be lost during subsequent runs of the code generator.
//!
//! version: 0.1.6
//!

#![allow(dead_code)]
#![allow(unused_imports)]

use yaserde_derive::{YaSerialize, YaDeserialize};
use std::io::{Read, Write};
use log::{warn, debug};

pub const SOAP_ENCODING: &str = "http://www.w3.org/2003/05/soap-encoding";

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
pub struct Header {}

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

impl std::error::Error for SoapFault {}

impl std::fmt::Display for SoapFault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (&self.fault_code, &self.fault_string) {
            (None, None) => Ok(()),
            (None, Some(fault_string)) => f.write_str(fault_string),
            (Some(fault_code), None) => f.write_str(fault_code),
            (Some(fault_code), Some(fault_string)) => {
                f.write_str(fault_code)?;
                f.write_str(": ")?;
                f.write_str(fault_string)
            }
        }
    }
}

pub type SoapResponse = Result<(reqwest::StatusCode, String), reqwest::Error>;

pub mod messages {
    use yaserde::{YaSerialize, YaDeserialize};
    use yaserde::de::from_str;
    use async_trait::async_trait;
    use yaserde::ser::to_string;
    use super::*;

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "GetXmlFeedMessage",
    )]
    pub struct GetXmlFeedMessage {
        #[yaserde(flatten, default)]
        pub get_xml_feed_request: types::GetXmlFeed,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "GetXmlFeedResponseMessage",
    )]
    pub struct GetXmlFeedResponseMessage {
        #[yaserde(flatten, default)]
        pub get_xml_feed_response: types::GetXmlFeedResponse,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "GetXmlFeedRequestHeader",
    )]
    pub struct GetXmlFeedRequestHeader {
        #[yaserde(flatten, default)]
        pub ath_header: types::AuthTokenHeader,
    }
}

pub mod types {
    use yaserde::{YaSerialize, YaDeserialize};
    use yaserde::de::from_str;
    use async_trait::async_trait;
    use yaserde::ser::to_string;
    use super::*;



    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "AuthTokenHeader",
    namespace = "nsi1: http://www.msn.com/webservices/spaces/v1/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    prefix = "nsi1",
    )]
    pub struct AuthTokenHeader {
        #[yaserde(rename = "Token", default)]
        pub token: String,
    }

    pub type GetXmlFeed = GetXmlFeedRequestType;





    pub type GetXmlFeedResponse = GetXmlFeedResponseType;


}

pub mod ports {
    use yaserde::{YaSerialize, YaDeserialize};
    use yaserde::de::from_str;
    use async_trait::async_trait;
    use yaserde::ser::to_string;
    use super::*;

    pub type GetXmlFeedMessage = messages::GetXmlFeedMessage;

    pub type GetXmlFeedResponseMessage = messages::GetXmlFeedResponseMessage;

    #[async_trait]
    pub trait SpaceServicePortType {
        async fn get_xml_feed(&self, get_xml_feed_message: GetXmlFeedMessage) -> Result<GetXmlFeedResponseMessage, Option<SoapFault>>;
    }
}




