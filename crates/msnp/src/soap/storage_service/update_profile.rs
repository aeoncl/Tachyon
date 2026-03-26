pub mod request {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::error::SoapMarshallError;
    use crate::soap::storage_service::get_profile::request::GetProfileMessageSoapEnvelope;
    use crate::soap::storage_service::headers::StorageServiceHeaders;
    use crate::soap::storage_service::msnstorage_datatypes::{ExpressionProfileAttributesType, Profile};
    use crate::soap::traits::xml::TryFromXml;

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
    default_namespace = "nsi1"
    )]
    pub struct UpdateProfileRequestType {
        #[yaserde(rename = "profile", prefix="nsi1")]
        pub profile: Profile,
        #[yaserde(rename = "profileAttributesToDelete", prefix="nsi1")]
        pub profile_attributes_to_delete: Option<ProfileAttributesToDelete>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "profileAttributesToDelete",
        namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
        prefix = "nsi1"
    )]
    pub struct ProfileAttributesToDelete {
        #[yaserde(rename = "ExpressionProfileAttributes", prefix="nsi1")]
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

    impl TryFromXml for UpdateProfileMessageSoapEnvelope {
        type Error = SoapMarshallError;

        fn try_from_xml(xml_str: &str) -> Result<Self, Self::Error> {
            yaserde::de::from_str::<Self>(&xml_str).map_err(|e| Self::Error::DeserializationError { message: e})
        }
    }

}

pub mod response {
    use yaserde::ser::to_string;
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::error::SoapMarshallError;
    use crate::soap::storage_service::fault::SoapFault;
    use crate::soap::storage_service::get_profile::response::GetProfileResponseMessageSoapEnvelope;
    use crate::soap::storage_service::headers::{AffinityCacheHeader, StorageServiceHeaders};
    use crate::soap::traits::xml::ToXml;

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
        pub fn new(cache_key: String) -> Self {

            let affinity_cache_header = AffinityCacheHeader{ cache_key: Some(cache_key)};

            let headers = StorageServiceHeaders{ storage_application: None, storage_user: None, affinity_cache: Some(affinity_cache_header) };

            UpdateProfileResponseMessageSoapEnvelope {
                body: SoapUpdateProfileResponseMessage::default(),
                header: headers,
            }
        }
    }

    impl ToXml for UpdateProfileResponseMessageSoapEnvelope {
        type Error = SoapMarshallError;
        fn to_xml(&self) -> Result<String, Self::Error>  {
            to_string(self).map_err(|e| SoapMarshallError::SerializationError { message: e})
        }

    }


}