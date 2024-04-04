pub mod request {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::space::space_ws_datatype_xsd::types::RefreshInformationType;
    use crate::soap::space::space_ws_wsdl::{Header, ports, SOAP_ENCODING};

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapGetXmlFeedMessage {
        #[yaserde(rename = "GetXmlFeed", default)]
        pub body: GetXmlFeedRequestType,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "GetXmlFeed",
    )]
    pub struct GetXmlFeedRequestType {
        #[yaserde(rename = "refreshInformation", default)]
        pub refresh_information: RefreshInformationType,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
    prefix = "soapenv"
    )]
    pub struct GetXmlFeedMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soapenv", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapGetXmlFeedMessage,
    }

    impl GetXmlFeedMessageSoapEnvelope {
        pub fn new(body: SoapGetXmlFeedMessage) -> Self {
            GetXmlFeedMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/spaces/v1/".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }


}

pub mod response {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::space::space_ws_datatype_xsd::types::{BorderType, ClientAreaType, ContactCardType, ElementsType, ElementType, LiveThemeType, SurfaceType, ThemeType};
    use crate::soap::space::space_ws_wsdl::{Header, ports, SoapFault};

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
    )]
    pub struct GetXmlFeedResponseType {
        #[yaserde(rename = "GetXmlFeedResult", default)]
        pub get_xml_feed_result: GetXmlFeedResultType,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "GetXmlFeedResultType",
    )]
    pub struct GetXmlFeedResultType {
        #[yaserde(rename = "contactCard", default)]
        pub contact_card: ContactCardType,
    }


    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
    prefix = "soapenv"
    )]
    pub struct GetXmlFeedResponseMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soapenv", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapGetXmlFeedResponseMessage,
    }

    impl GetXmlFeedResponseMessageSoapEnvelope {
        pub fn new(body: SoapGetXmlFeedResponseMessage) -> Self {
            GetXmlFeedResponseMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/spaces/v1/".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

}