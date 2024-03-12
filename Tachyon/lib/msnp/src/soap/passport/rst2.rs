pub mod request {

    #[cfg(test)]
    mod tests {
        use yaserde::de::from_str;

        use super::RST2RequestMessageSoapEnvelope;

        #[test]
        fn test_rst2_request() {
    
            let request = "<?xml version=\"1.0\" encoding=\"UTF-8\"?><s:Envelope xmlns:s=\"http://www.w3.org/2003/05/soap-envelope\" xmlns:wsse=\"http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-secext-1.0.xsd\" xmlns:saml=\"urn:oasis:names:tc:SAML:1.0:assertion\" xmlns:wsp=\"http://schemas.xmlsoap.org/ws/2004/09/policy\" xmlns:wsu=\"http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-utility-1.0.xsd\" xmlns:wsa=\"http://www.w3.org/2005/08/addressing\" xmlns:wssc=\"http://schemas.xmlsoap.org/ws/2005/02/sc\" xmlns:wst=\"http://schemas.xmlsoap.org/ws/2005/02/trust\"><s:Header><wsa:Action s:mustUnderstand=\"1\">http://schemas.xmlsoap.org/ws/2005/02/trust/RST/Issue</wsa:Action><wsa:To s:mustUnderstand=\"1\">HTTPS://127.0.0.1:80//RST2.srf</wsa:To><wsa:MessageID>1650180844</wsa:MessageID><ps:AuthInfo xmlns:ps=\"http://schemas.microsoft.com/Passport/SoapServices/PPCRL\" Id=\"PPAuthInfo\"><ps:HostingApp>{7108E71A-9926-4FCB-BCC9-9A9D3F32E423}</ps:HostingApp><ps:BinaryVersion>5</ps:BinaryVersion><ps:UIVersion>1</ps:UIVersion><ps:Cookies></ps:Cookies><ps:RequestParams>AQAAAAIAAABsYwQAAAAyMDYw</ps:RequestParams></ps:AuthInfo><wsse:Security><wsse:UsernameToken wsu:Id=\"user\"><wsse:Username>test@homeserver.org</wsse:Username><wsse:Password>passwd</wsse:Password></wsse:UsernameToken><wsu:Timestamp Id=\"Timestamp\"><wsu:Created>2022-04-17T09:34:04Z</wsu:Created><wsu:Expires>2022-04-17T09:39:04Z</wsu:Expires></wsu:Timestamp></wsse:Security></s:Header><s:Body><ps:RequestMultipleSecurityTokens xmlns:ps=\"http://schemas.microsoft.com/Passport/SoapServices/PPCRL\" Id=\"RSTS\"><wst:RequestSecurityToken Id=\"RST0\"><wst:RequestType>http://schemas.xmlsoap.org/ws/2005/02/trust/Issue</wst:RequestType><wsp:AppliesTo><wsa:EndpointReference><wsa:Address>http://Passport.NET/tb</wsa:Address></wsa:EndpointReference></wsp:AppliesTo></wst:RequestSecurityToken><wst:RequestSecurityToken Id=\"RST1\"><wst:RequestType>http://schemas.xmlsoap.org/ws/2005/02/trust/Issue</wst:RequestType><wsp:AppliesTo><wsa:EndpointReference><wsa:Address>messengerclear.live.com</wsa:Address></wsa:EndpointReference></wsp:AppliesTo><wsp:PolicyReference URI=\"MBI_KEY_OLD\"></wsp:PolicyReference></wst:RequestSecurityToken><wst:RequestSecurityToken Id=\"RST2\"><wst:RequestType>http://schemas.xmlsoap.org/ws/2005/02/trust/Issue</wst:RequestType><wsp:AppliesTo><wsa:EndpointReference><wsa:Address>messenger.msn.com</wsa:Address></wsa:EndpointReference></wsp:AppliesTo><wsp:PolicyReference URI=\"?id=507\"></wsp:PolicyReference></wst:RequestSecurityToken><wst:RequestSecurityToken Id=\"RST3\"><wst:RequestType>http://schemas.xmlsoap.org/ws/2005/02/trust/Issue</wst:RequestType><wsp:AppliesTo><wsa:EndpointReference><wsa:Address>messengersecure.live.com</wsa:Address></wsa:EndpointReference></wsp:AppliesTo><wsp:PolicyReference URI=\"MBI_SSL\"></wsp:PolicyReference></wst:RequestSecurityToken><wst:RequestSecurityToken Id=\"RST4\"><wst:RequestType>http://schemas.xmlsoap.org/ws/2005/02/trust/Issue</wst:RequestType><wsp:AppliesTo><wsa:EndpointReference><wsa:Address>contacts.msn.com</wsa:Address></wsa:EndpointReference></wsp:AppliesTo><wsp:PolicyReference URI=\"MBI\"></wsp:PolicyReference></wst:RequestSecurityToken><wst:RequestSecurityToken Id=\"RST5\"><wst:RequestType>http://schemas.xmlsoap.org/ws/2005/02/trust/Issue</wst:RequestType><wsp:AppliesTo><wsa:EndpointReference><wsa:Address>storage.msn.com</wsa:Address></wsa:EndpointReference></wsp:AppliesTo><wsp:PolicyReference URI=\"MBI\"></wsp:PolicyReference></wst:RequestSecurityToken><wst:RequestSecurityToken Id=\"RST6\"><wst:RequestType>http://schemas.xmlsoap.org/ws/2005/02/trust/Issue</wst:RequestType><wsp:AppliesTo><wsa:EndpointReference><wsa:Address>sup.live.com</wsa:Address></wsa:EndpointReference></wsp:AppliesTo><wsp:PolicyReference URI=\"MBI\"></wsp:PolicyReference></wst:RequestSecurityToken></ps:RequestMultipleSecurityTokens></s:Body></s:Envelope>";
    
            let test : RST2RequestMessageSoapEnvelope = from_str(&request).unwrap();
    
            let test2 = test.header.security.username_token;
            
            //TODO add assertions
            let test3 = 1;
        }  
    }


    use yaserde_derive::{YaDeserialize, YaSerialize};

    use crate::soap::{error::SoapError, traits::xml::TryFromXml};

    use super::shared::{AppliesTo, SecurityHeader};

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "s: http://www.w3.org/2003/05/soap-envelope",
    namespace = "wsse: http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-secext-1.0.xsd",
    namespace = "saml: urn:oasis:names:tc:SAML:1.0:assertion",
    namespace = "wsp: http://schemas.xmlsoap.org/ws/2004/09/policy",
    namespace = "wsu: http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-utility-1.0.xsd",
    namespace = "wsa: http://www.w3.org/2005/08/addressing"
    namespace = "wssc: http://schemas.xmlsoap.org/ws/2005/02/sc"
    namespace = "wst: http://schemas.xmlsoap.org/ws/2005/02/trust"
    prefix = "s"
    )]
    pub struct RST2RequestMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "s")]
        pub header: RST2RequestMessageHeader,

        #[yaserde(rename = "Body", prefix = "s")]
        pub body: RST2RequestMessageBody,
    }

    impl TryFromXml for RST2RequestMessageSoapEnvelope {

        type Error = SoapError;
        
        fn try_from_xml(xml_str: &str) -> Result<Self, Self::Error> {
            yaserde::de::from_str::<Self>(&xml_str).map_err(|e| Self::Error::DeserializationError { message: e})
        }
    }


    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        namespace = "wsse: http://schemas.xmlsoap.org/ws/2004/09/policy",
        prefix = "wsse"
    )]

    pub struct RST2RequestMessageHeader {
        #[yaserde(rename = "Security", prefix = "wsse")]
        pub security: SecurityHeader,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct RST2RequestMessageBody {
        #[yaserde(rename = "RequestMultipleSecurityTokens", prefix = "ps")]
        pub request_multiple_security_tokens: RequestMultipleSecurityTokens,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        namespace = "ps: http://schemas.microsoft.com/Passport/SoapServices/PPCRL",
        prefix = "ps"
    )]

    pub struct RequestMultipleSecurityTokens {
        #[yaserde(rename = "Id", attribute)]
        pub id: String,

        #[yaserde(rename = "RequestSecurityToken", prefix = "wst")]
        pub request_security_tokens: Vec<RequestSecurityToken>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        namespace = "wst: http://schemas.xmlsoap.org/ws/2005/02/trust",
        namespace = "wsp: http://schemas.xmlsoap.org/ws/2004/09/policy",
        prefix = "wst"
    )]

    pub struct RequestSecurityToken {
        #[yaserde(rename = "Id", attribute)]
        pub id: String,

        #[yaserde(rename = "RequestType", prefix = "wst")]
        pub request_type: String,

        #[yaserde(rename = "AppliesTo", prefix = "wsp")]
        pub applies_to: AppliesTo,
    }
}

pub mod response {

    pub mod factory {

        use chrono::{DateTime, Days, Local};

        use crate::{
            shared::models::uuid::Uuid,
            soap::passport::rst2::shared::{
                AppliesTo, EndpointReference, Reference, SecurityHeader, Timestamp,
            },
        };

        use super::{
            ActionHeader, BinarySecurityToken, CipherData, EncryptedData, EncryptionMethod,
            KeyIdentifier, KeyInfo, Lifetime, PassportProperties, RST2ResponseMessageBody,
            RST2ResponseMessageHeader, RST2ResponseMessageSoapEnvelope,
            RequestSecurityTokenResponse, RequestSecurityTokenResponseCollection,
            RequestedAttachedReference, RequestedProofToken, RequestedSecurityToken,
            RequestedTokenReference, SecurityTokenReference, ServerInfo,
        };

        pub struct RST2ResponseFactory;

        impl RST2ResponseFactory {

            pub fn get_rst2_success_response(
                matrix_token: String,
                msn_addr: String,
                uuid: Uuid,
            ) -> RST2ResponseMessageSoapEnvelope {
                let action = ActionHeader::new(
                    String::from("Action"),
                    1,
                    String::from("http://schemas.xmlsoap.org/ws/2005/02/trust/RSTR/Issue"),
                );
                let to = ActionHeader::new(
                    String::from("To"),
                    1,
                    String::from("http://schemas.xmlsoap.org/ws/2004/08/addressing/role/anonymous"),
                );

                let now = Local::now();

                let tomorrow = now.checked_add_days(Days::new(1)).expect("day to be added");

                let timestamp = Timestamp::new(
                    String::from("TS"),
                    now.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                    tomorrow.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                );
                let security = SecurityHeader {
                    must_understand: Some(1),
                    timestamp,
                    username_token: None,
                };

                let server_info = ServerInfo {
                    path: String::from("Live1"),
                    rolling_upgrade_state: String::from("ExclusiveNew"),
                    loc_version: 0,
                    server_time: now.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                    body: String::from("XYZPPLOGN1A23 2017.09.28.12.44.07"),
                };

                let pp = PassportProperties::new(
                    uuid.get_puid().to_string(),
                    uuid.to_hex_cid(),
                    msn_addr,
                    server_info,
                );
                let header = RST2ResponseMessageHeader {
                    action,
                    to,
                    security,
                    passport_properties: pp,
                };

                let body = RST2ResponseMessageBody {
                    request_security_token_response_collection: RST2ResponseFactory::get_tokens(
                        matrix_token,
                    ),
                };
                RST2ResponseMessageSoapEnvelope { header, body }
            }

            fn get_tokens(matrix_token: String) -> RequestSecurityTokenResponseCollection {
                let mut request_security_token_response = Vec::new();
                request_security_token_response.push(RST2ResponseFactory::get_legacy_token());

                let mut tokens = RST2ResponseFactory::get_relevant_tokens(matrix_token);
                request_security_token_response.append(&mut tokens);

                RequestSecurityTokenResponseCollection {
                    request_security_token_response
                }
            }

            fn get_legacy_token() -> RequestSecurityTokenResponse {
                let applies_to = AppliesTo {
                    endpoint_reference: EndpointReference {
                        address: String::from("http://Passport.NET/tb"),
                    },
                };
                let now = Local::now();
                let tomorrow = now.checked_add_days(Days::new(1)).expect("day to be added");
                let lifetime = Lifetime {
                    created: now.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                    expires: tomorrow.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                };
                let requested_security_token= RequestedSecurityToken{ binary_security_token: None, encrypted_data: Some(EncryptedData{ id: String::from("BinaryDAToken0"), data_type: String::from("http://www.w3.org/2001/04/xmlenc#Element"), encryption_method: EncryptionMethod{ algorithm: String::from("http://www.w3.org/2001/04/xmlenc#tripledes-cbc") }, key_info: KeyInfo{ key_name: String::from("http://Passport.NET/STS") }, cipher_data: CipherData{ cipher_value: String::from("Cap26AQZrSyMm2SwwTyJKyqLR9/S+vQWQsaBc5Mv7PwtQDMzup/udOOMMvSu99R284pmiD3IepBXrEMLK5rLrXAf2A6vrP6vYuGA45GCqQdoxusHZcjt9P2B8WyCTVT2cM8jtGqGIfRlU/4WzOLxNrDJwDfOsmilduGAGZfvRPW7/jyXXrnGK7/PWkymX4YDD+ygJfMrPAfvAprvw/HVE6tutKVc9cViTVYy8oHjosQlb8MKn3vKDW1O2ZWQUc47JPl7DkjQaanfNBGe6CL7K1nr6Z/jy7Ay7MjV+KQehmvphSEmCzLrpB4WWn2PdpdTrOcDj+aJfWHeGL4sIPwEKgrKnTQg9QD8CCsm5wew9P/br39OuIfsC6/PFBEHmVThqj0aMxYLRD4K2GoRay6Ab7NftoIP5dnFnclfRxETAoNpTPE2F5Q669QySrdXxBpBSk8GLmdCDMlhiyzSiByrhFQaZRcH8n9i+i289otYuJQ7xPyP19KwT4CRyOiIlh3DSdlBfurMwihQGxN2spU7P4MwckrDKeOyYQhvNm/XWId/oXBqpHbo2yRPiOwL9p1J4AxA4RaJuh77vyhn2lFQaxPDqZd5A8RJjpb2NE2N3UncKLW7GAangdoLbRDMqt51VMZ0la+b/moL61fKvFXinKRHc7PybrG3MWzgXxO/VMKAuXOsB9XnOgl2A524cgiwyg==") } })};
                let requested_attached_reference = RequestedAttachedReference {
                    security_token_reference: SecurityTokenReference {
                        reference: Reference {
                            uri: String::from("2jmj7l5rSw0yVb/vlWAYkK/YBwk="),
                        },
                    },
                };
                let requested_unattached_reference = RequestedAttachedReference {
                    security_token_reference: SecurityTokenReference {
                        reference: Reference {
                            uri: String::from("2jmj7l5rSw0yVb/vlWAYkK/YBwk="),
                        },
                    },
                };

                let requested_proof_token = RequestedProofToken {
                    binary_secret: String::from("tgoPVK67sU36fQKlGLMgWgTXp7oiaQgE"),
                };

                RequestSecurityTokenResponse {
                    token_type: String::from("urn:passport:legacy"),
                    applies_to,
                    lifetime,
                    requested_security_token,
                    requested_token_reference: None,
                    requested_attached_reference,
                    requested_unattached_reference,
                    requested_proof_token,
                }
            }

            fn get_relevant_tokens(matrix_token: String) -> Vec<RequestSecurityTokenResponse> {
                let mut out: Vec<RequestSecurityTokenResponse> = Vec::new();
                let addresses = [
                    "messengerclear.live.com",
                    "messenger.msn.com",
                    "messengersecure.live.com",
                    "contacts.msn.com",
                    "storage.msn.com",
                    "sup.live.com",
                ];
                let now = Local::now();
                let tomorrow = now.checked_add_days(Days::new(1)).expect("day to be added");

                for i in 0..6 {
                    out.push(RST2ResponseFactory::get_relevant_token(
                        matrix_token.clone(),
                        addresses[i].to_string(),
                        i.try_into().unwrap(),
                        &now,
                        &tomorrow,
                    ));
                }

                out
            }

            fn get_relevant_token(
                matrix_token: String,
                address: String,
                count: i32,
                created: &DateTime<Local>,
                expires: &DateTime<Local>,
            ) -> RequestSecurityTokenResponse {
                let applies_to = AppliesTo {
                    endpoint_reference: EndpointReference { address: address },
                };
                let lifetime = Lifetime {
                    created: created.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                    expires: expires.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                };

                let token_id = format!("Compact{}", count + 1);
                let token_uri = format!("#{}", token_id);
                let token = format!("t={}", matrix_token);

                let binary_security_token = BinarySecurityToken {
                    id: token_id.clone(),
                    token,
                };
                let requested_security_token = RequestedSecurityToken {
                    binary_security_token: Some(binary_security_token),
                    encrypted_data: None,
                };
                let requested_attached_reference = RequestedAttachedReference {
                    security_token_reference: SecurityTokenReference {
                        reference: Reference {
                            uri: String::from("/DaESnwwMVTTpRTZEoNqUW/Md0k="),
                        },
                    },
                };
                let requested_unattached_reference = RequestedAttachedReference {
                    security_token_reference: SecurityTokenReference {
                        reference: Reference {
                            uri: String::from("/DaESnwwMVTTpRTZEoNqUW/Md0k="),
                        },
                    },
                };
                let requested_token_reference = RequestedTokenReference {
                    key_identifier: KeyIdentifier {
                        value_type: String::from("urn:passport:compact"),
                    },
                    reference: Reference {
                        uri: token_uri.clone(),
                    },
                };
                let requested_proof_token = RequestedProofToken {
                    binary_secret: String::from("tgoPVK67sU36fQKlGLMgWgTXp7oiaQgE"),
                };

                RequestSecurityTokenResponse {
                    token_type: String::from("urn:passport:compact"),
                    applies_to,
                    lifetime,
                    requested_security_token,
                    requested_token_reference: Some(requested_token_reference),
                    requested_attached_reference,
                    requested_unattached_reference,
                    requested_proof_token,
                }
            }

            pub fn get_auth_error_response() -> String {
                //TODO Use the ps-fault xsd
                let now = Local::now();
                let server_time = now.format("%Y-%m-%dT%H:%M:%SZ").to_string();

                let out = "<?xml version=\"1.0\" encoding=\"utf-8\" ?><S:Envelope xmlns:S=\"http://www.w3.org/2003/05/soap-envelope\" xmlns:wsse=\"http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-secext-1.0.xsd\" xmlns:wsu=\"http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-utility-1.0.xsd\" xmlns:wst=\"http://schemas.xmlsoap.org/ws/2005/02/trust\" xmlns:psf=\"http://schemas.microsoft.com/Passport/SoapServices/SOAPFault\"><S:Header><psf:pp xmlns:psf=\"http://schemas.microsoft.com/Passport/SoapServices/SOAPFault\"><psf:serverVersion>1</psf:serverVersion><psf:authstate>0x80048800</psf:authstate><psf:reqstatus>0x80048821</psf:reqstatus><psf:serverInfo Path=\"Live1\" RollingUpgradeState=\"ExclusiveNew\" LocVersion=\"0\" ServerTime=\"{{server_time}}\" BuildVersion=\"16.0.28426.6\">XYZPPLOGN1A23 2017.09.28.12.44.07</psf:serverInfo><psf:cookies/><psf:response/></psf:pp></S:Header><S:Body><S:Fault><S:Code><S:Value>S:Sender</S:Value><S:Subcode><S:Value>wst:FailedAuthentication</S:Value></S:Subcode></S:Code><S:Reason><S:Text xml:lang=\"en-US\">Authentication Failure</S:Text></S:Reason><S:Detail><psf:error><psf:value>0x80048821</psf:value><psf:internalerror><psf:code>0x80041012</psf:code><psf:text>The entered and stored passwords do not match.&#x000D;&#x000A;</psf:text></psf:internalerror></psf:error></S:Detail></S:Fault></S:Body></S:Envelope>";
                out.replace("{{server_time}}", &server_time)
            }
        }
    }
    #[cfg(test)]
    mod tests {
        use tests::factory::RST2ResponseFactory;
        use yaserde::ser::to_string;

        use crate::shared::models::uuid::Uuid;
        use crate::soap::passport::rst2::response::*;
        use crate::soap::passport::rst2::shared::{EndpointReference, Timestamp};

        #[test]
        fn test_factory() {
          let test =  RST2ResponseFactory::get_rst2_success_response("t0k3n".to_string(),"aeon@test.com".to_string(), Uuid::new());
            //TODO add assertions
            println!("{}", to_string(&test).unwrap());
        }

        #[test]
        fn test_rst2_response() {
            let action = ActionHeader::new(String::from("Action"), 1, String::from("http://schemas.xmlsoap.org/ws/2005/02/trust/RSTR/Issue"));
            let to = ActionHeader::new(String::from("To"), 1, String::from("http://schemas.xmlsoap.org/ws/2004/08/addressing/role/anonymous"));
    
            let timestamp = Timestamp::new(String::from("TS"), String::from("created"), String::from("expires"));
            let security = SecurityHeader{must_understand: Some(1), timestamp, username_token: None};
    
    
            let server_info = ServerInfo { path: String::from("Live1"), rolling_upgrade_state: String::from("ExclusiveNew"), loc_version: 0, server_time: String::from("servertime"), body: String::from("XYZPPLOGN1A23 2017.09.28.12.44.07")};
           
    
            let pp = PassportProperties::new(String::from("puid"), String::from("hex_cid") , String::from("msn_addr"),server_info);
            let header = RST2ResponseMessageHeader {action, to, security, passport_properties: pp };
    
    
            let mut request_security_token_response = Vec::new();
    
            let applies_to = AppliesTo { endpoint_reference: EndpointReference { address: String::from("http://Passport.NET/tb") } };
            let lifetime = Lifetime{created: String::from("created"), expires: String::from("expires")};
            let requested_security_token= RequestedSecurityToken{ binary_security_token: None, encrypted_data: Some(EncryptedData{ id: String::from("BinaryDAToken0"), data_type: String::from("http://www.w3.org/2001/04/xmlenc#Element"), encryption_method: EncryptionMethod{ algorithm: String::from("http://www.w3.org/2001/04/xmlenc#tripledes-cbc") }, key_info: KeyInfo{ key_name: String::from("http://Passport.NET/STS") }, cipher_data: CipherData{ cipher_value: String::from("Cap26AQZrSyMm2SwwTyJKyqLR9/S+vQWQsaBc5Mv7PwtQDMzup/udOOMMvSu99R284pmiD3IepBXrEMLK5rLrXAf2A6vrP6vYuGA45GCqQdoxusHZcjt9P2B8WyCTVT2cM8jtGqGIfRlU/4WzOLxNrDJwDfOsmilduGAGZfvRPW7/jyXXrnGK7/PWkymX4YDD+ygJfMrPAfvAprvw/HVE6tutKVc9cViTVYy8oHjosQlb8MKn3vKDW1O2ZWQUc47JPl7DkjQaanfNBGe6CL7K1nr6Z/jy7Ay7MjV+KQehmvphSEmCzLrpB4WWn2PdpdTrOcDj+aJfWHeGL4sIPwEKgrKnTQg9QD8CCsm5wew9P/br39OuIfsC6/PFBEHmVThqj0aMxYLRD4K2GoRay6Ab7NftoIP5dnFnclfRxETAoNpTPE2F5Q669QySrdXxBpBSk8GLmdCDMlhiyzSiByrhFQaZRcH8n9i+i289otYuJQ7xPyP19KwT4CRyOiIlh3DSdlBfurMwihQGxN2spU7P4MwckrDKeOyYQhvNm/XWId/oXBqpHbo2yRPiOwL9p1J4AxA4RaJuh77vyhn2lFQaxPDqZd5A8RJjpb2NE2N3UncKLW7GAangdoLbRDMqt51VMZ0la+b/moL61fKvFXinKRHc7PybrG3MWzgXxO/VMKAuXOsB9XnOgl2A524cgiwyg==") } })};
            
            let requested_attached_reference= RequestedAttachedReference { security_token_reference: SecurityTokenReference{ reference: Reference{ uri: String::from("2jmj7l5rSw0yVb/vlWAYkK/YBwk=") } } };
            let requested_unattached_reference= RequestedAttachedReference { security_token_reference: SecurityTokenReference{ reference: Reference{ uri: String::from("2jmj7l5rSw0yVb/vlWAYkK/YBwk=") } } };
            let requested_proof_token = RequestedProofToken{ binary_secret: String::from("tgoPVK67sU36fQKlGLMgWgTXp7oiaQgE") };
            
            request_security_token_response.push(RequestSecurityTokenResponse{ token_type: String::from("urn:passport:legacy"), applies_to, lifetime, requested_security_token, requested_token_reference: None, requested_attached_reference, requested_unattached_reference, requested_proof_token });
            
            let request_security_token_response_collection = RequestSecurityTokenResponseCollection { request_security_token_response};
            let body = RST2ResponseMessageBody { request_security_token_response_collection };
            let env = RST2ResponseMessageSoapEnvelope {header, body};
            
            //TODO assert something ???
            println!("{}", to_string(&env).unwrap());
        }

    }
    
    use yaserde_derive::{YaDeserialize, YaSerialize};

    use super::shared::{AppliesTo, Reference, SecurityHeader};

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "S: http://www.w3.org/2003/05/soap-envelope",
    namespace = "wsse: http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-secext-1.0.xsd",
    namespace = "wsu: http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-utility-1.0.xsd",
    namespace = "wsa: http://www.w3.org/2005/08/addressing"
    prefix = "S"
    )]

    pub struct RST2ResponseMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "S")]
        header: RST2ResponseMessageHeader,

        #[yaserde(rename = "Body", prefix = "S")]
        body: RST2ResponseMessageBody,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct RST2ResponseMessageHeader {
        #[yaserde(rename = "Action", prefix = "wsa")]
        action: ActionHeader,
        #[yaserde(rename = "To", prefix = "wsa")]
        to: ActionHeader,

        #[yaserde(rename = "Security", prefix = "wsse")]
        security: SecurityHeader,

        #[yaserde(rename = "pp", prefix = "psf")]
        passport_properties: PassportProperties,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct ActionHeader {
        #[yaserde(rename = "S", prefix = "xmlns", attribute)]
        s_ns: String,

        #[yaserde(rename = "wsu", prefix = "xmlns", attribute)]
        wsu_ns: String,

        #[yaserde(rename = "wsa", prefix = "xmlns", attribute)]
        wsa_ns: String,

        #[yaserde(rename = "Id", prefix = "wsu", attribute)]
        id: String,

        #[yaserde(rename = "mustUnderstand", prefix = "S", attribute)]
        must_understand: i32,

        #[yaserde(text)]
        body: String,
    }

    impl ActionHeader {
        pub fn new(id: String, must_understand: i32, body: String) -> ActionHeader {
            ActionHeader{ s_ns: String::from("http://www.w3.org/2003/05/soap-envelope"), wsu_ns: String::from("http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-utility-1.0.xsd"), wsa_ns:String::from("http://www.w3.org/2005/08/addressing"), id, must_understand, body }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        prefix = "psf",
        namespace = "psf: http://schemas.microsoft.com/Passport/SoapServices/SOAPFault"
    )]

    pub struct PassportProperties {
        //#[yaserde(rename = "psf", prefix = "xmlns", attribute)]
        //psf_ns: String,
        #[yaserde(rename = "serverVersion", prefix = "psf")]
        server_version: i32,

        #[yaserde(rename = "PUID", prefix = "psf")]
        puid: String,

        #[yaserde(rename = "configVersion", prefix = "psf")]
        config_version: String,

        #[yaserde(rename = "uiVersion", prefix = "psf")]
        ui_version: String,

        #[yaserde(rename = "mobileConfigVersion", prefix = "psf")]
        mobile_config_version: String,

        #[yaserde(rename = "appDataVersion", prefix = "psf")]
        mapp_data_version: i32,

        #[yaserde(rename = "authstate", prefix = "psf")]
        authstate: String,

        #[yaserde(rename = "reqstatus", prefix = "psf")]
        reqstatus: String,

        #[yaserde(rename = "serverInfo", prefix = "psf")]
        server_info: ServerInfo,

        #[yaserde(rename = "cookies", prefix = "psf")]
        cookies: Option<String>,

        #[yaserde(rename = "browserCookies", prefix = "psf")]
        browser_cookies: ArrayOfBrowserCookies,

        #[yaserde(rename = "credProperties", prefix = "psf")]
        cred_properties: ArrayOfCredProperty,

        #[yaserde(rename = "extProperties", prefix = "psf", child)]
        ext_properties: ArrayOfExtProperty,

        #[yaserde(rename = "response", prefix = "psf")]
        response: Option<String>,
    }

    impl PassportProperties {
        pub fn new(
            puid: String,
            hex_cid: String,
            msn_addr: String,
            server_info: ServerInfo,
        ) -> PassportProperties {
            let browser_cookies = vec![
                BrowserCookie {
                    name: String::from("MH"),
                    url: String::from("http://www.msn.com"),
                    body: String::from(
                        "MSFT; path=/; domain=.msn.com; expires=Wed, 30-Dec-2037 16:00:00 GMT",
                    ),
                },
                BrowserCookie {
                    name: String::from("MHW"),
                    url: String::from("http://www.msn.com"),
                    body: String::from(
                        "; path=/; domain=.msn.com; expires=Thu, 30-Oct-1980 16:00:00 GMT",
                    ),
                },
                BrowserCookie {
                    name: String::from("MH"),
                    url: String::from("http://www.live.com"),
                    body: String::from(
                        "MSFT; path=/; domain=.live.com; expires=Wed, 30-Dec-2037 16:00:00 GMT",
                    ),
                },
                BrowserCookie {
                    name: String::from("MHW"),
                    url: String::from("http://www.live.com"),
                    body: String::from(
                        "; path=/; domain=.live.com; expires=Thu, 30-Oct-1980 16:00:00 GMT",
                    ),
                },
            ];

            let cred_properties = vec![
                CredProperty {
                    name: String::from("MainBrandID"),
                    body: String::from("MSFT"),
                },
                CredProperty {
                    name: String::from("BrandIDList"),
                    body: String::new(),
                },
                CredProperty {
                    name: String::from("IsWinLiveUser"),
                    body: String::from("true"),
                },
                CredProperty {
                    name: String::from("CID"),
                    body: hex_cid,
                },
                CredProperty {
                    name: String::from("AuthMembername"),
                    body: msn_addr,
                },
                CredProperty {
                    name: String::from("Country"),
                    body: String::from("US"),
                },
                CredProperty {
                    name: String::from("Language"),
                    body: String::from("1033"),
                },
                CredProperty {
                    name: String::from("FirstName"),
                    body: String::from("John"),
                },
                CredProperty {
                    name: String::from("LastName"),
                    body: String::from("Doe"),
                },
                CredProperty {
                    name: String::from("ChildFlags"),
                    body: String::from("00000001"),
                },
                CredProperty {
                    name: String::from("Flags"),
                    body: String::from("40100643"),
                },
                CredProperty {
                    name: String::from("FlagsV2"),
                    body: String::from("00000000"),
                },
                CredProperty {
                    name: String::from("IP"),
                    body: String::from("127.0.0.1"),
                },
                CredProperty {
                    name: String::from("AssociatedForStrongAuth"),
                    body: String::from("0"),
                },
            ];

            let ext_properties = vec![
                ExtProperty {
                    name: String::from("ANON"),
                    expiry: Some(String::from("Wed, 30-Dec-2037 16:00:00 GMT")),
                    domains: Some(String::from("bing.com;atdmt.com")),
                    ignore_remember_me: Some(false),
                    body: String::from("A=B97FB2EE7DB4CE0D0D5B8107FFFFFFFF&amp;E=1542&amp;W=1"),
                },
                ExtProperty { 
                    name: String::from("NAP"), 
                    expiry: Some(String::from("Wed, 30-Dec-2037 16:00:00 GMT")), 
                    domains: Some(String::from("bing.com;atdmt.com")), 
                    ignore_remember_me: Some(false), 
                    body: String::from("V=1.9&amp;E=14e8&amp;C=uT838e-8kV7Jbm-HqQel-ETkvE7QSUGh6ywMjZQ9JJyYtNKxtdfCBw&amp;W=1") 
                },
                ExtProperty {
                    name: String::from("LastUsedCredType"),
                    expiry: None,
                    domains: None,
                    ignore_remember_me: None,
                    body: String::from("1"),
                },
                ExtProperty {
                    name: String::from("WebCredType"),
                    expiry: None,
                    domains: None,
                    ignore_remember_me: None,
                    body: String::from("1"),
                },
                ExtProperty {
                    name: String::from("CID"),
                    expiry: None,
                    domains: None,
                    ignore_remember_me: None,
                    body: String::from("%CID%"),
                }
            ];

            PassportProperties {
                server_version: 1,
                puid,
                config_version: String::from("16.000.26889.00"),
                ui_version: String::from("3.100.2179.0"),
                mobile_config_version: String::from("16.000.26208.0"),
                mapp_data_version: 1,
                authstate: String::from("0x48803"),
                reqstatus: String::from("0x0"),
                server_info,
                cookies: Some(String::new()),
                browser_cookies: ArrayOfBrowserCookies { browser_cookies },
                cred_properties: ArrayOfCredProperty { cred_properties },
                ext_properties: ArrayOfExtProperty { ext_properties },
                response: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct BrowserCookie {
        #[yaserde(rename = "Name", attribute)]
        name: String,

        #[yaserde(rename = "URL", attribute)]
        url: String,

        #[yaserde(text)]
        body: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct CredProperty {
        #[yaserde(rename = "Name", attribute)]
        name: String,

        #[yaserde(text)]
        body: String,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct ExtProperty {
        #[yaserde(rename = "Name", attribute)]
        name: String,

        #[yaserde(rename = "Expiry", attribute)]
        expiry: Option<String>,

        #[yaserde(rename = "Domains", attribute)]
        domains: Option<String>,

        #[yaserde(rename = "IgnoreRememberMe", attribute)]
        ignore_remember_me: Option<bool>,

        #[yaserde(text)]
        body: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct ServerInfo {
        #[yaserde(rename = "Path", attribute)]
        path: String,

        #[yaserde(rename = "RollingUpgradeState", attribute)]
        rolling_upgrade_state: String,

        #[yaserde(rename = "LocVersion", attribute)]
        loc_version: i32,

        #[yaserde(rename = "ServerTime", attribute)]
        server_time: String,

        #[yaserde(text)]
        body: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct ArrayOfBrowserCookies {
        #[yaserde(rename = "browserCookie", prefix = "psf")]
        browser_cookies: Vec<BrowserCookie>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct ArrayOfCredProperty {
        #[yaserde(rename = "credProperty", prefix = "psf")]
        cred_properties: Vec<CredProperty>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct ArrayOfExtProperty {
        #[yaserde(rename = "extProperty", prefix = "psf", child)]
        ext_properties: Vec<ExtProperty>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct RST2ResponseMessageBody {
        #[yaserde(rename = "RequestSecurityTokenResponseCollection", prefix = "wst")]
        request_security_token_response_collection: RequestSecurityTokenResponseCollection,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        prefix = "wst",
        namespace = "S: http://www.w3.org/2003/05/soap-envelope",
        namespace = "wst: http://schemas.xmlsoap.org/ws/2005/02/trust",
        namespace = "wsse: http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-secext-1.0.xsd",
        namespace = "wsu: http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-utility-1.0.xsd",
        namespace = "saml: urn:oasis:names:tc:SAML:1.0:assertion",
        namespace = "wsp: http://schemas.xmlsoap.org/ws/2004/09/policy",
        namespace = "psf: http://schemas.microsoft.com/Passport/SoapServices/SOAPFault"
    )]
    pub struct RequestSecurityTokenResponseCollection {
        #[yaserde(rename = "RequestSecurityTokenResponse", prefix = "wst")]
        request_security_token_response: Vec<RequestSecurityTokenResponse>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct RequestSecurityTokenResponse {
        #[yaserde(rename = "TokenType", prefix = "wst")]
        token_type: String,

        #[yaserde(rename = "AppliesTo", prefix = "wsp")]
        applies_to: AppliesTo,

        #[yaserde(rename = "Lifetime", prefix = "wst")]
        lifetime: Lifetime,

        #[yaserde(rename = "RequestedSecurityToken", prefix = "wst")]
        requested_security_token: RequestedSecurityToken,

        #[yaserde(rename = "RequestedTokenReference", prefix = "wst")]
        requested_token_reference: Option<RequestedTokenReference>,

        #[yaserde(rename = "RequestedAttachedReference", prefix = "wst")]
        requested_attached_reference: RequestedAttachedReference,

        #[yaserde(rename = "RequestedUnattachedReference", prefix = "wst")]
        requested_unattached_reference: RequestedAttachedReference,

        #[yaserde(rename = "RequestedProofToken", prefix = "wst")]
        requested_proof_token: RequestedProofToken,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct RequestedTokenReference {
        #[yaserde(rename = "KeyIdentifier", prefix = "wsse")]
        key_identifier: KeyIdentifier,

        #[yaserde(rename = "Reference", prefix = "wsse")]
        reference: Reference,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct KeyIdentifier {
        #[yaserde(rename = "ValueType", attribute)]
        value_type: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct RequestedAttachedReference {
        #[yaserde(rename = "SecurityTokenReference", prefix = "wsse")]
        security_token_reference: SecurityTokenReference,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SecurityTokenReference {
        #[yaserde(rename = "Reference", prefix = "wsse")]
        reference: Reference,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct RequestedProofToken {
        #[yaserde(rename = "BinarySecret", prefix = "wst")]
        binary_secret: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct Lifetime {
        #[yaserde(rename = "Created", prefix = "wsu")]
        created: String,

        #[yaserde(rename = "Expires", prefix = "wsu")]
        expires: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct RequestedSecurityToken {
        #[yaserde(rename = "BinarySecurityToken", prefix = "wst")]
        binary_security_token: Option<BinarySecurityToken>,

        #[yaserde(rename = "EncryptedData")]
        encrypted_data: Option<EncryptedData>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct BinarySecurityToken {
        #[yaserde(rename = "Id", prefix = "wsse", attribute)]
        id: String,

        #[yaserde(rename = "Token", text)]
        token: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        prefix = "wesh",
        namespace = "wesh: http://www.w3.org/2001/04/xmlenc#",
        default_namespace = "wesh"
    )]
    pub struct EncryptedData {
        #[yaserde(rename = "Id", prefix = "wesh", attribute)]
        id: String,
        #[yaserde(rename = "Type", prefix = "wesh", attribute)]
        data_type: String,
        #[yaserde(rename = "EncryptionMethod", prefix = "wesh")]
        encryption_method: EncryptionMethod,

        #[yaserde(rename = "KeyInfo", prefix = "ds")]
        key_info: KeyInfo,

        #[yaserde(rename = "CipherData", prefix = "wesh")]
        cipher_data: CipherData,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        prefix = "wesh",
        namespace = "wesh: http://www.w3.org/2001/04/xmlenc#",
        default_namespace = "wesh"
    )]
    pub struct EncryptionMethod {
        #[yaserde(rename = "Algorithm", prefix = "wesh", attribute)]
        algorithm: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(prefix = "ds", namespace = "ds: http://www.w3.org/2000/09/xmldsig#")]
    pub struct KeyInfo {
        #[yaserde(rename = "KeyName", prefix = "ds")]
        key_name: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        prefix = "wesh",
        namespace = "wesh: http://www.w3.org/2001/04/xmlenc#",
        default_namespace = "wesh"
    )]
    pub struct CipherData {
        #[yaserde(rename = "CipherValue", prefix = "wesh")]
        cipher_value: String,
    }
}

pub mod shared {
    use yaserde_derive::{YaDeserialize, YaSerialize};

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct Reference {
        #[yaserde(rename = "URI", attribute)]
        pub(crate) uri: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SecurityHeader {
        #[yaserde(rename = "mustUnderstand", prefix = "S", attribute)]
        pub must_understand: Option<i32>,

        #[yaserde(rename = "UsernameToken", prefix = "wsse")]
        pub username_token: Option<UsernameToken>,

        #[yaserde(rename = "Timestamp", prefix = "wsu")]
        pub timestamp: Timestamp,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        namespace = "wsse: http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-secext-1.0.xsd",
        prefix = "wsse"
    )]
    pub struct UsernameToken {
        #[yaserde(rename = "Username", prefix = "wsse")]
        pub username: String,
        #[yaserde(rename = "Password", prefix = "wsse")]
        pub password: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        namespace = "wsu: http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-utility-1.0.xsd",
        prefix = "wsu"
    )]
    pub struct Timestamp {
        #[yaserde(rename = "wsu", prefix = "xmlns", attribute)]
        pub wsu_ns: String,

        #[yaserde(rename = "Id", prefix = "wsu", attribute)]
        pub id: String,

        #[yaserde(rename = "Created", prefix = "wsu")]
        pub created: String,

        #[yaserde(rename = "Expires", prefix = "wsu")]
        pub expires: String,
    }

    impl Timestamp {
        pub fn new(id: String, created: String, expires: String) -> Self {
            Self { wsu_ns: String::from("http://docs.oasis-open.org/wss/2004/01/oasis-200401-wss-wssecurity-utility-1.0.xsd"), id, created, expires }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        prefix = "wst",
        namespace = "wst: http://schemas.xmlsoap.org/ws/2005/02/trust",
        namespace = "wsa: http://www.w3.org/2005/08/addressing"
    )]
    pub struct AppliesTo {
        #[yaserde(rename = "EndpointReference", prefix = "wsa")]
        pub endpoint_reference: EndpointReference,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        prefix = "wsa",
        namespace = "wsa: http://www.w3.org/2005/08/addressing"
    )]
    pub struct EndpointReference {
        #[yaserde(rename = "Address", prefix = "wsa")]
        pub address: String,
    }
}
