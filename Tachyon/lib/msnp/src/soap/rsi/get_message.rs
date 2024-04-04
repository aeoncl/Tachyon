pub mod request {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::rsi::service_header::ServiceHeader;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapGetMessageMessage {
        #[yaserde(rename = "GetMessage", default)]
        pub body: GetMessageRequestType,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "GetMessage",
    )]
    pub struct GetMessageRequestType {
        #[yaserde(rename = "messageId", default)]
        pub message_id: String,
        #[yaserde(rename = "alsoMarkAsRead", default)]
        pub also_mark_as_read: bool,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soapenv"
    )]
    pub struct GetMessageMessageSoapEnvelope {
        pub header: Option<ServiceHeader>,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapGetMessageMessage,
    }

    impl GetMessageMessageSoapEnvelope {
        pub fn new(body: SoapGetMessageMessage) -> Self {
            GetMessageMessageSoapEnvelope {
                body,
                header: None,
            }
        }
    }


}

pub mod response {

    #[cfg(test)]
    mod tests {
        #[test]
        fn ser_test() {



        }

    }

    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::rsi::service_header::ServiceHeader;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapGetMessageResponseMessage {
        #[yaserde(rename = "GetMessageResponse", default)]
        pub body: GetMessageResponse
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "GetMessageResponse",
    )]
    pub struct GetMessageResponse {
        #[yaserde(rename = "GetMessageResult", default)]
        pub get_message_result: String,
    }


    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soapenv"
    )]
    pub struct GetMessageResponseMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: Option<ServiceHeader>,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapGetMessageResponseMessage,
    }

    impl GetMessageResponseMessageSoapEnvelope {
        pub fn new(body: SoapGetMessageResponseMessage) -> Self {
            GetMessageResponseMessageSoapEnvelope {
                body,
                header: None,
            }
        }
    }



}