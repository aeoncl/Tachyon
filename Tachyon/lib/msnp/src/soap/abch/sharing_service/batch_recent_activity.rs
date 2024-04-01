pub mod request {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::abch::msnab_sharingservice::{SOAP_ENCODING};
    use crate::soap::abch::request_header::RequestHeaderContainer;
    use crate::soap::abch::msnab_datatypes::{EntityHandle, Locale};

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapGetBatchRecentActivityMessage {
        #[yaserde(rename = "GetBatchRecentActivity", default)]
        pub body: GetBatchRecentActivityRequestType,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "GetBatchRecentActivityRequestType")]
    pub struct GetBatchRecentActivityRequestType {
        #[yaserde(rename = "entityHandle", default)]
        pub entity_handle: EntityHandle,
        #[yaserde(rename = "locales", default)]
        pub locales: Locale,
        #[yaserde(rename = "count", default)]
        pub count: i32,
        #[yaserde(rename = "templateTypes", default)]
        pub template_types: TemplateTypes,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "templateTypes")]
    pub struct TemplateTypes {
        #[yaserde(rename = "string", default)]
        pub string: Vec<String>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    prefix = "soap"
    )]
    pub struct GetBatchRecentActivityMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<RequestHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapGetBatchRecentActivityMessage,
    }

    impl GetBatchRecentActivityMessageSoapEnvelope {
        pub fn new(body: SoapGetBatchRecentActivityMessage) -> Self {
            GetBatchRecentActivityMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
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
    use crate::soap::abch::sharing_service::contact_recent_activity::response::{Templates};
    use crate::soap::abch::msnab_faults::SoapFault;
    use crate::soap::abch::msnab_sharingservice::{SOAP_ENCODING};
    use crate::soap::abch::request_header::RequestHeaderContainer;
    use crate::soap::abch::msnab_datatypes::Activities;

    #[cfg(test)]
    mod tests {

    }


    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapGetBatchRecentActivityResponseMessage {
        #[yaserde(rename = "GetBatchRecentActivityResponseMessage", default)]
        pub body: GetBatchRecentActivityResultType,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "GetBatchRecentActivityResultType")]
    pub struct GetBatchRecentActivityResultType {
        #[yaserde(rename = "Activities", default)]
        pub activities: Activities,
        #[yaserde(rename = "Templates", default)]
        pub templates: Templates,
        #[yaserde(rename = "FeedUrl", default)]
        pub feed_url: Option<String>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    prefix = "soap"
    )]
    pub struct GetBatchRecentActivityResponseMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<RequestHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapGetBatchRecentActivityResponseMessage,
    }

    impl GetBatchRecentActivityResponseMessageSoapEnvelope {
        pub fn new(body: SoapGetBatchRecentActivityResponseMessage) -> Self {
            GetBatchRecentActivityResponseMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }


}

