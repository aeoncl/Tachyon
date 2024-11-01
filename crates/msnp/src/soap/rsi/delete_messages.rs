pub mod request {
    use yaserde_derive::{YaDeserialize, YaSerialize};

    use crate::soap::error::SoapMarshallError;
    use crate::soap::rsi::get_message::request::GetMessageMessageSoapEnvelope;
    use crate::soap::rsi::service_header::ServiceHeader;
    use crate::soap::traits::xml::TryFromXml;

    #[cfg(test)]
    mod tests {
        use yaserde::de::from_str;

        use crate::soap::rsi::delete_messages::request::DeleteMessagesSoapEnvelope;

        #[test]
        fn deser_test() {

            let msg = "<?xml version=\"1.0\" encoding=\"utf-8\"?><soap:Envelope xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\" xmlns:soap=\"http://schemas.xmlsoap.org/soap/envelope/\"><soap:Header><PassportCookie xmlns=\"http://www.hotmail.msn.com/ws/2004/09/oim/rsi\"><t>t0ken</t><p>ppppp</p></PassportCookie></soap:Header><soap:Body><DeleteMessages xmlns=\"http://www.hotmail.msn.com/ws/2004/09/oim/rsi\"><messageIds><messageId>id</messageId></messageIds></DeleteMessages></soap:Body></soap:Envelope>";

            let deser = from_str::<DeleteMessagesSoapEnvelope>(msg).expect("To work");

            assert_eq!("t0ken", deser.header.expect("header to be here").passport_cookie.t);
            assert_eq!("id", deser.body.body.message_ids.message_id[0])
        }


    }


    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapDeleteMessagesMessage {
        #[yaserde(rename = "DeleteMessages", default)]
        pub body: DeleteMessagesRequestType
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "DeleteMessages",
    namespace = "nsi1: http://www.hotmail.msn.com/ws/2004/09/oim/rsi",
    default_namespace = "nsi1"
    )]
    pub struct DeleteMessagesRequestType {
        #[yaserde(rename = "messageIds", default)]
        pub message_ids: MessageIds,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "messageIds",
    namespace = "nsi1: http://www.hotmail.msn.com/ws/2004/09/oim/rsi"
    )]
    pub struct MessageIds {
        #[yaserde(rename = "messageId",  prefix="nsi1")]
        pub message_id: Vec<String>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soapenv"
    )]
    pub struct DeleteMessagesSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: Option<ServiceHeader>,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapDeleteMessagesMessage,
    }

    impl TryFromXml for DeleteMessagesSoapEnvelope {

        type Error = SoapMarshallError;

        fn try_from_xml(xml_str: &str) -> Result<Self, Self::Error> {
            yaserde::de::from_str::<Self>(&xml_str).map_err(|e| Self::Error::DeserializationError { message: e})
        }
    }

    impl DeleteMessagesSoapEnvelope {
        pub fn new(body: SoapDeleteMessagesMessage, header: Option<ServiceHeader>) -> Self {
            DeleteMessagesSoapEnvelope {
                body,
                header: None,
            }
        }
    }


}

pub mod response {
    use yaserde::ser::to_string;
    use yaserde_derive::{YaDeserialize, YaSerialize};

    use crate::soap::error::SoapMarshallError;
    use crate::soap::rsi::get_message::response::GetMessageResponseMessageSoapEnvelope;
    use crate::soap::rsi::service_header::ServiceHeader;
    use crate::soap::traits::xml::ToXml;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapDeleteMessagesResponseMessage {
        #[yaserde(rename = "DeleteMessagesResponse", default)]
        pub body: DeleteMessagesResponseType
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "DeleteMessagesResponse",
    namespace = "nsi1: http://www.hotmail.msn.com/ws/2004/09/oim/rsi",
    default_namespace = "nsi1"
    )]
    pub struct DeleteMessagesResponseType {}


    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soapenv"
    )]
    pub struct DeleteMessagesResponseSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: Option<ServiceHeader>,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapDeleteMessagesResponseMessage,
    }


    impl ToXml for DeleteMessagesResponseSoapEnvelope {
        type Error = SoapMarshallError;

        fn to_xml(&self) -> Result<String, Self::Error>  {
            to_string(self).map_err(|e| SoapMarshallError::SerializationError { message: e})
        }
    }


    impl DeleteMessagesResponseSoapEnvelope {
        pub fn new() -> Self {
            
            
            DeleteMessagesResponseSoapEnvelope {
                body: SoapDeleteMessagesResponseMessage{
                    body: DeleteMessagesResponseType{},
                },
                header: None,
            }
        }
    }
}