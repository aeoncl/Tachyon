pub mod request {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::storage_service::headers::StorageServiceHeaders;
    use crate::soap::storage_service::msnstorage_datatypes::Profile;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapCreateProfileMessage {
        #[yaserde(rename = "CreateProfile", default)]
        pub body: CreateProfileRequestType,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "CreateProfile",
    namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
    prefix = "nsi1"
    )]
    pub struct CreateProfileRequestType {
        #[yaserde(rename = "profile", prefix = "nsi1")]
        pub profile: Profile,
    }


    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soapenv"
    )]
    pub struct CreateProfileMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: StorageServiceHeaders,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapCreateProfileMessage,
    }

    impl CreateProfileMessageSoapEnvelope {
        pub fn new(body: SoapCreateProfileMessage) -> Self {
            CreateProfileMessageSoapEnvelope {
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

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapCreateProfileResponseMessage {
        #[yaserde(rename = "CreateProfileResponse", default)]
        pub body: CreateProfileResponse,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,

    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "CreateProfileResponse",
    namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    prefix = "nsi1",
    )]
    pub struct CreateProfileResponse {
        #[yaserde(rename = "CreateProfileResult", prefix = "nsi1")]
        pub create_profile_result: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soapenv"
    )]
    pub struct CreateProfileResponseMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: StorageServiceHeaders,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapCreateProfileResponseMessage,
    }

    impl CreateProfileResponseMessageSoapEnvelope {
        pub fn new(body: SoapCreateProfileResponseMessage) -> Self {
            CreateProfileResponseMessageSoapEnvelope {
                body,
                header: StorageServiceHeaders::default(),
            }
        }
    }

}