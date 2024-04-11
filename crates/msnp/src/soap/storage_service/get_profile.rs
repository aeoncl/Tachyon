pub mod request {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::storage_service::headers::StorageServiceHeaders;
    use crate::soap::storage_service::msnstorage_datatypes::{Handle, ProfileAttributes};

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapGetProfileMessage {
        #[yaserde(rename = "GetProfile", default)]
        pub body: GetProfileRequestType,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }


    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "GetProfile",
    namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
    prefix = "nsi1"
    )]
    pub struct GetProfileRequestType {
        #[yaserde(rename = "profileHandle", default)]
        pub profile_handle: Handle,
        #[yaserde(rename = "profileAttributes", default)]
        pub profile_attributes: ProfileAttributes,
    }


    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soapenv"
    )]
    pub struct GetProfileMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: StorageServiceHeaders,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapGetProfileMessage,
    }

    impl GetProfileMessageSoapEnvelope {
        pub fn new(body: SoapGetProfileMessage) -> Self {
            GetProfileMessageSoapEnvelope {
                body,
                header: StorageServiceHeaders::default(),
            }
        }
    }



}

pub mod response {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::storage_service::fault::SoapFault;
    use crate::soap::storage_service::headers::StorageServiceHeaders;
    use crate::soap::storage_service::msnstorage_datatypes::ExpressionProfile;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapGetProfileResponseMessage {
        #[yaserde(rename = "GetProfileResponse", default)]
        pub body: GetProfileResponse,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,

    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "GetProfileResponse",
    namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
    prefix = "nsi1",
    )]
    pub struct GetProfileResponse {
        #[yaserde(rename = "GetProfileResult", default)]
        pub get_profile_result: GetProfileResultType,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "GetProfileResultType",
    )]
    pub struct GetProfileResultType {
        #[yaserde(rename = "ResourceID", default)]
        pub resource_id: String,
        #[yaserde(rename = "DateModified", default)]
        pub date_modified: String,
        #[yaserde(rename = "ExpressionProfile", default)]
        pub expression_profile: ExpressionProfile,
    }


    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soapenv"
    )]
    pub struct GetProfileResponseMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: StorageServiceHeaders,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapGetProfileResponseMessage,
    }

    impl GetProfileResponseMessageSoapEnvelope {
        pub fn new(body: SoapGetProfileResponseMessage) -> Self {
            GetProfileResponseMessageSoapEnvelope {
                body,
                header: StorageServiceHeaders::default(),
            }
        }
    }


}