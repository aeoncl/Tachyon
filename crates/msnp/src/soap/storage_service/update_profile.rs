pub mod request {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::storage_service::headers::StorageServiceHeaders;
    use crate::soap::storage_service::msnstorage_datatypes::{ExpressionProfileAttributesType, Profile};

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapUpdateProfileMessage {
        #[yaserde(rename = "UpdateProfile", default)]
        pub body: UpdateProfileRequestType,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "UpdateProfile",
    namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
    prefix = "nsi1"
    )]
    pub struct UpdateProfileRequestType {
        #[yaserde(rename = "profile", default)]
        pub profile: Profile,
        #[yaserde(rename = "profileAttributesToDelete", default)]
        pub profile_attributes_to_delete: Option<ProfileAttributesToDelete>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "profileAttributesToDelete",
    )]
    pub struct ProfileAttributesToDelete {
        #[yaserde(rename = "ExpressionProfileAttributes", default)]
        pub expression_profile_attributes: ExpressionProfileAttributesType,
    }



    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soapenv"
    )]
    pub struct UpdateProfileMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: StorageServiceHeaders,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapUpdateProfileMessage,
    }

    impl UpdateProfileMessageSoapEnvelope {
        pub fn new(body: SoapUpdateProfileMessage) -> Self {
            UpdateProfileMessageSoapEnvelope {
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
    pub struct SoapUpdateProfileResponseMessage {
        #[yaserde(rename = "UpdateProfileResponseMessage", default)]
        pub body: UpdateProfileResponseMessage,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,

    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "UpdateProfileResponse",
    namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
    prefix = "nsi1"
    )]
    pub struct UpdateProfileResponseMessage {
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soapenv"
    )]
    pub struct UpdateProfileResponseMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: StorageServiceHeaders,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapUpdateProfileResponseMessage,
    }

    impl UpdateProfileResponseMessageSoapEnvelope {
        pub fn new(body: SoapUpdateProfileResponseMessage) -> Self {
            UpdateProfileResponseMessageSoapEnvelope {
                body,
                header: StorageServiceHeaders::default(),
            }
        }
    }
}