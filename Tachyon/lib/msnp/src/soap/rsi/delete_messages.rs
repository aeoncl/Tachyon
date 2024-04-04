pub mod request {

    #[cfg(test)]
    mod tests {
        use yaserde::de::from_str;
        use crate::soap::rsi::delete_messages::request::DeleteMessagesMessageSoapEnvelope;

        #[test]
        fn deser_test() {

            let msg = "<?xml version=\"1.0\" encoding=\"utf-8\"?><soap:Envelope xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\" xmlns:soap=\"http://schemas.xmlsoap.org/soap/envelope/\"><soap:Header><PassportCookie xmlns=\"http://www.hotmail.msn.com/ws/2004/09/oim/rsi\"><t>t0ken</t><p>ppppp</p></PassportCookie></soap:Header><soap:Body><DeleteMessages xmlns=\"http://www.hotmail.msn.com/ws/2004/09/oim/rsi\"><messageIds><messageId>id</messageId></messageIds></DeleteMessages></soap:Body></soap:Envelope>";

            let deser = from_str::<DeleteMessagesMessageSoapEnvelope>(msg).expect("To work");

            assert_eq!("t0ken", deser.header.expect("header to be here").passport_cookie.t);
            assert_eq!("id", deser.body.body.message_ids.message_id[0])
        }


    }


    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::rsi::service_header::ServiceHeader;




    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapDeleteMessagesMessage {
        #[yaserde(rename = "DeleteMessages", default)]
        pub body: DeleteMessagesRequestType
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "DeleteMessages",
    namespace = "nsi1: http://www.hotmail.msn.com/ws/2004/09/oim/rsi",
    prefix = "nsi1"
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
    pub struct DeleteMessagesMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: Option<ServiceHeader>,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapDeleteMessagesMessage,
    }

    impl DeleteMessagesMessageSoapEnvelope {
        pub fn new(body: SoapDeleteMessagesMessage, header: Option<ServiceHeader>) -> Self {
            DeleteMessagesMessageSoapEnvelope {
                body,
                header: None,
            }
        }
    }


}

pub mod response {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::rsi::service_header::ServiceHeader;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapDeleteMessagesResponseMessage {
        #[yaserde(rename = "DeleteMessagesResponse", default)]
        pub body: DeleteMessagesResponseType
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "DeleteMessagesResponseType",
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



    impl DeleteMessagesResponseSoapEnvelope {
        pub fn new(body: SoapDeleteMessagesResponseMessage) -> Self {
            DeleteMessagesResponseSoapEnvelope {
                body,
                header: None,
            }
        }
    }
}