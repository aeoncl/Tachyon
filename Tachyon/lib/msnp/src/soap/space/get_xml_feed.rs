pub mod request {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::space::headers::{AuthTokenHeader, SpaceHeaders};
    use crate::soap::space::space_ws_datatype_xsd::RefreshInformationType;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapGetXmlFeedMessage {
        #[yaserde(rename = "GetXmlFeed", default)]
        pub body: GetXmlFeedRequestType
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "GetXmlFeed",
    namespace = "nsi1: http://www.msn.com/webservices/spaces/v1/",
    prefix = "nsi1"
    )]
    pub struct GetXmlFeedRequestType {
        #[yaserde(rename = "refreshInformation", prefix="nsi1")]
        pub refresh_information: RefreshInformationType,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soapenv"
    )]
    pub struct GetXmlFeedMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: Option<SpaceHeaders>,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapGetXmlFeedMessage,
    }

    impl GetXmlFeedMessageSoapEnvelope {
        pub fn new(body: SoapGetXmlFeedMessage) -> Self {
            GetXmlFeedMessageSoapEnvelope {
                body,
                header: None,
            }
        }
    }


}

pub mod response {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::space::fault::SoapFault;
    use crate::soap::space::headers::SpaceHeaders;
    use crate::soap::space::space_ws_datatype_xsd::ContactCardType;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapGetXmlFeedResponseMessage {
        #[yaserde(rename = "GetXmlFeedResponse", default)]
        pub body: GetXmlFeedResponseType,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,

    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "GetXmlFeedResponse",
    namespace = "nsi1: http://www.msn.com/webservices/spaces/v1/",
    prefix = "nsi1"
    )]
    pub struct GetXmlFeedResponseType {
        #[yaserde(rename = "GetXmlFeedResult", default)]
        pub get_xml_feed_result: GetXmlFeedResultType,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "GetXmlFeedResultType",
    namespace = "nsi1: http://www.msn.com/webservices/spaces/v1/",
    prefix = "nsi1"
    )]
    pub struct GetXmlFeedResultType {
        #[yaserde(rename = "contactCard", prefix="nsi1")]
        pub contact_card: ContactCardType,
    }


    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soapenv"
    )]
    pub struct GetXmlFeedResponseMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: Option<SpaceHeaders>,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapGetXmlFeedResponseMessage,
    }

    impl GetXmlFeedResponseMessageSoapEnvelope {
        pub fn new(body: SoapGetXmlFeedResponseMessage) -> Self {
            GetXmlFeedResponseMessageSoapEnvelope {
                body,
                header: None,
            }
        }
    }

}