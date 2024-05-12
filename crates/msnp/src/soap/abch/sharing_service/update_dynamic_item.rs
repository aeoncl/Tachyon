pub mod request {
    use yaserde_derive::{YaDeserialize, YaSerialize};

    use crate::soap::abch::msnab_datatypes::DynamicItems;
    use crate::soap::abch::request_header::RequestHeaderContainer;
    use crate::soap::abch::sharing_service::manage_wl_connection::request::ManageWLConnectionMessageSoapEnvelope;
    use crate::soap::error::SoapMarshallError;
    use crate::soap::traits::xml::TryFromXml;

    #[cfg(test)]
    mod tests {
        use yaserde::de::from_str;

        use crate::soap::abch::ab_service::ab_contact_add::request::AbcontactAddMessageSoapEnvelope;

        #[test]
        fn test_update_dynamic_item_request() {
            let request = "<?xml version=\"1.0\" encoding=\"utf-8\"?><soap:Envelope xmlns:soap=\"http://schemas.xmlsoap.org/soap/envelope/\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xmlns:xsd=\"http://www.w3.org2001/XMLSchema\" xmlns:soapenc=\"http://schemas.xmlsoap.org/soap/encoding/\"><soap:Header><ABApplicationHeader xmlns=\"http://www.msn.com/webservices/AddressBook\"><ApplicationId>AAD9B99B-58E6-4F23-B975-D9EC1F9EC24A</ApplicationId><IsMigration>false</IsMigration><PartnerScenario>RoamingIdentityChanged</PartnerScenario><CacheKey>12r1:8nBBE6vX1J4uPKajtbem5XBIblimCwAhIziAeEAwYD0AMiaztryWvcZthkN9oX_pl2scBKXfKvRvuWKYdHUNuRkgiyV9rzcDpnDIDiM6vdcEB6d82wjjnL4TAFAjc5X8i-C9mNfQujUk470Pfz9qbWfK6ANcEtygDb-oWsYVfEBrxl6geTUg9tGT7yCIsls7ECcLyqwsROuAbWCrued_VPKiUgSIvqG8gaA</CacheKey></ABApplicationHeader><ABAuthHeader xmlns=\"http://www.msn.com/webservices/AddressBook\"><ManagedGroupRequest>false</ManagedGroupRequest><TicketToken>t=0bfusc4t3dT0k3n</TicketToken></ABAuthHeader></soap:Header><soap:Body><UpdateDynamicItem xmlns=\"http://www.msn.com/webservices/AddressBook\"><abId>00000000-0000-0000-0000-000000000000</abId><dynamicItems><DynamicItem xsi:type=\"PassportDynamicItem\"><Type>Passport</Type><PassportName>aeoncl@matrix.org</PassportName><Notifications><NotificationData><StoreService><Info><Handle><Id>0</Id><Type>Profile</Type><ForeignId>MyProfile</ForeignId></Handle><InverseRequired>false</InverseRequired><IsBot>false</IsBot></Info><Changes /><LastChange>0001-01-01T00:00:00</LastChange><Deleted>false</Deleted></StoreService><Status>Exist Access</Status><Gleam>false</Gleam><InstanceId>0</InstanceId></NotificationData></Notifications><Changes>Notifications</Changes></DynamicItem></dynamicItems></UpdateDynamicItem></soap:Body></soap:Envelope>";

            let request_deserialized : AbcontactAddMessageSoapEnvelope = from_str(request).unwrap();

            //TODO assertions
        }

    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapUpdateDynamicItemMessage {
        #[yaserde(rename = "UpdateDynamicItem", default)]
        pub body: UpdateDynamicItemRequestType,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "UpdateDynamicItemRequestType",
    namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    prefix = "nsi1",
    default_namespace="nsi1")]
    pub struct UpdateDynamicItemRequestType {
        #[yaserde(rename = "abId", prefix="nsi1")]
        pub ab_id: String,
        #[yaserde(rename = "dynamicItems", prefix="nsi1")]
        pub dynamic_items: DynamicItems,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soap"
    )]
    pub struct UpdateDynamicItemMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<RequestHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapUpdateDynamicItemMessage,
    }

    impl UpdateDynamicItemMessageSoapEnvelope {
        pub fn new(body: SoapUpdateDynamicItemMessage) -> Self {
            UpdateDynamicItemMessageSoapEnvelope {
                body,
                header: None,
            }
        }
    }

    impl TryFromXml for UpdateDynamicItemMessageSoapEnvelope {
        type Error = SoapMarshallError;

        fn try_from_xml(xml_str: &str) -> Result<Self, Self::Error> {
            yaserde::de::from_str::<Self>(&xml_str).map_err(|e| Self::Error::DeserializationError { message: e})
        }
    }


}

#[cfg(test)]
pub mod response {
    use yaserde::ser::to_string;
    use yaserde_derive::{YaDeserialize, YaSerialize};

    use crate::soap::abch::msnab_faults::SoapFault;
    use crate::soap::abch::service_header::ServiceHeaderContainer;
    use crate::soap::abch::sharing_service::manage_wl_connection::response::ManageWLConnectionResponseMessageSoapEnvelope;
    use crate::soap::error::SoapMarshallError;
    use crate::soap::traits::xml::ToXml;

    mod tests {
        use yaserde::ser::to_string;

        use crate::soap::abch::sharing_service::update_dynamic_item::response::{SoapUpdateDynamicItemResponseMessage, UpdateDynamicItemResponseMessage, UpdateDynamicItemResponseMessageSoapEnvelope};

        #[test]
        fn test_update_dynamic_item_response() {
            let body_content = UpdateDynamicItemResponseMessage{ update_dynamic_item_response: None };
            let body = SoapUpdateDynamicItemResponseMessage { body: body_content, fault: None };
            let response =  UpdateDynamicItemResponseMessageSoapEnvelope{ header: None, body };
            let response_serialized = to_string(&response).unwrap();

            //TODO assertions

        }

    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapUpdateDynamicItemResponseMessage {
        #[yaserde(rename = "UpdateDynamicItemResponse", default)]
        pub body: UpdateDynamicItemResponseMessage,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "UpdateDynamicItemResponseMessage")]
    pub struct UpdateDynamicItemResponseMessage {
        #[yaserde(default)]
        pub update_dynamic_item_response: Option<String>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soap"
    )]
    pub struct UpdateDynamicItemResponseMessageSoapEnvelope {
        pub header: Option<ServiceHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapUpdateDynamicItemResponseMessage,
    }

    impl UpdateDynamicItemResponseMessageSoapEnvelope {
        pub fn new(cache_key: &str) -> Self {
            let body_content = UpdateDynamicItemResponseMessage{ update_dynamic_item_response: None };
            let body = SoapUpdateDynamicItemResponseMessage { body: body_content, fault: None };
            Self{ header: Some(ServiceHeaderContainer::new(cache_key)), body }
        }
    }

    impl ToXml for UpdateDynamicItemResponseMessageSoapEnvelope {
        type Error = SoapMarshallError;
        fn to_xml(&self) -> Result<String, Self::Error>  {
            to_string(self).map_err(|e| SoapMarshallError::SerializationError { message: e})
        }

    }

}