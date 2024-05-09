use crate::soap::abch::ab_service::ab_find_contacts_paged::request::AbfindContactsPagedMessageSoapEnvelope;
use crate::soap::error::SoapMarshallError;
use crate::soap::traits::xml::TryFromXml;

pub mod request {

    #[cfg(test)]
    mod tests {
        use yaserde::de::from_str;
        use crate::soap::abch::ab_service::ab_find_contacts_paged::request::AbfindContactsPagedMessageSoapEnvelope;

        #[test]
        fn test_find_contacts_paged_request() {
            let request_body = r#"<?xml version="1.0" encoding="utf-8"?>
                                    <soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/"
                                    	xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
                                    	xmlns:xsd="http://www.w3.org/2001/XMLSchema"
                                    	xmlns:soapenc="http://schemas.xmlsoap.org/soap/encoding/">
                                    	<soap:Header>
                                    		<ABApplicationHeader xmlns="http://www.msn.com/webservices/AddressBook">
                                    			<ApplicationId>AAD9B99B-58E6-4F23-B975-D9EC1F9EC24A</ApplicationId>
                                    			<IsMigration>false</IsMigration>
                                    			<PartnerScenario>Initial</PartnerScenario>
                                    			<CacheKey>cachekey</CacheKey>
                                    		</ABApplicationHeader>
                                    		<ABAuthHeader xmlns="http://www.msn.com/webservices/AddressBook">
                                    			<ManagedGroupRequest>false</ManagedGroupRequest>
                                    			<TicketToken>t=0bfus4t3d_t0k3n</TicketToken>
                                    		</ABAuthHeader>
                                    	</soap:Header>
                                    	<soap:Body>
                                    		<ABFindContactsPaged xmlns="http://www.msn.com/webservices/AddressBook">
                                    			<filterOptions>
                                    				<DeltasOnly>true</DeltasOnly>
                                    				<LastChanged>2023-05-21T19:49:28Z</LastChanged>
                                    				<ContactFilter>
                                    					<IncludeHiddenContacts>true</IncludeHiddenContacts>
                                    				</ContactFilter>
                                    			</filterOptions>
                                    			<abView>MessengerClient8</abView>
                                    			<extendedContent>AB AllGroups CircleResult</extendedContent>
                                    		</ABFindContactsPaged>
                                    	</soap:Body>
                                    </soap:Envelope>
                                    "#;

            let r : AbfindContactsPagedMessageSoapEnvelope = from_str(&request_body).unwrap();

            let header = &r.header.unwrap();
            assert_eq!(header.ab_auth_header.ticket_token, String::from("t=0bfus4t3d_t0k3n"));
            assert_eq!(header.application_header.partner_scenario, String::from("Initial"));
        }

    }

    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::abch::msnab_datatypes::{AbHandleType, FilterOptionsType, PageContextType};
    use crate::soap::abch::request_header::RequestHeaderContainer;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Body",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    prefix = "soap",
    default_namespace="soap"
    )]
    pub struct SoapAbfindContactsPagedMessage {
        #[yaserde(rename = "ABFindContactsPaged", default)]
        pub body: AbfindContactsPagedRequestType,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABFindContactsPagedRequestType",
    namespace = "soap: http://www.msn.com/webservices/AddressBook",
    prefix = "soap",
    default_namespace="soap"
    )]
    pub struct AbfindContactsPagedRequestType {
        #[yaserde(rename = "filterOptions", prefix="soap")]
        pub filter_options: FilterOptionsType,
        #[yaserde(rename = "abView", prefix="soap")]
        pub ab_view: String,
        #[yaserde(rename = "extendedContent", prefix="soap")]
        pub extended_content: String,
        #[yaserde(rename = "abHandle", prefix="soap")]
        pub ab_handle: Option<AbHandleType>,
        #[yaserde(rename = "pageContext", prefix="soap")]
        pub page_context: Option<PageContextType>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soap"
    )]
    pub struct AbfindContactsPagedMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<RequestHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAbfindContactsPagedMessage,
    }

    impl AbfindContactsPagedMessageSoapEnvelope {
        pub fn new(body: SoapAbfindContactsPagedMessage) -> Self {
            AbfindContactsPagedMessageSoapEnvelope {
                body,
                header: None,
            }
        }
    }

}

impl TryFromXml for AbfindContactsPagedMessageSoapEnvelope {
    type Error = SoapMarshallError;

    fn try_from_xml(xml_str: &str) -> Result<Self, Self::Error> {
        yaserde::de::from_str::<Self>(&xml_str).map_err(|e| Self::Error::DeserializationError { message: e})
    }
}

pub mod response {

    #[cfg(test)]
    mod tests {
        use yaserde::ser::to_string;
        use crate::soap::abch::ab_service::ab_find_contacts_paged::response::{Ab, AbfindContactsPagedResponse, AbfindContactsPagedResponseMessageSoapEnvelope, AbfindContactsPagedResultType, Groups, SoapAbfindContactsPagedResponseMessage};
        use crate::soap::abch::msnab_datatypes::{AbInfoType, AddressBookType, ArrayOfContactType, CircleResultType, ContactType, GroupType};
        use crate::soap::abch::service_header::{ServiceHeader, ServiceHeaderContainer};

        #[test]
        fn test_find_contacts_paged_response() {
            let service_header = ServiceHeader{ version: String::from("15.01.1408.0000"), cache_key: Some(String::from("cache_key")), cache_key_changed: Some(true), preferred_host_name: Some(String::from("localhost")), session_id: None };
            let service_header_container = ServiceHeaderContainer{ service_header };

            let ab_info_type = AbInfoType{ migrated_to: None, beta_status: None, name: None, owner_puid: 0, owner_cid: 0, owner_email:None, f_default: false, joined_namespace: false, is_bot: false, is_parent_managed: false, account_tier: None, account_tier_last_changed: String::new(), profile_version: 0, subscribe_external_partner: false, notify_external_partner: false, address_book_type: AddressBookType::Individual, messenger_application_service_created: None, is_beta_migrated: None, last_relevance_update: None};
            let ab = Ab{ ab_id: String::from("new_ab_id"), ab_info: ab_info_type, last_change: String::new(), dynamic_item_last_changed: String::new(), recent_activity_item_last_changed: None, create_date: String::new(), properties_changed: String::new() };

            let contact_array : Vec<ContactType> = Vec::new();
            let group_array : Vec<GroupType> = Vec::new();

            let array_of_contact = ArrayOfContactType{ contact: contact_array };
            let groups = Groups{ group: group_array };

            let circle_result = CircleResultType{ circles: None, circle_ticket: String::from("&lt;?xml version=\"1.0\" encoding=\"utf-16\"?&gt;&lt;SignedTicket xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\" ver=\"1\" keyVer=\"1\"&gt;&lt;Data&gt;PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0idXRmLTE2Ij8+DQo8VGlja2V0IHhtbG5zOnhzaT0iaHR0cDovL3d3dy53My5vcmcvMjAwMS9YTUxTY2hlbWEtaW5zdGFuY2UiIHhtbG5zOnhzZD0iaHR0cDovL3d3dy53My5vcmcvMjAwMS9YTUxTY2hlbWEiPg0KICA8VFM+MDAwMC0wMS0wMVQwMDowMDowMDwvVFM+DQogIDxDSUQ+LTc3NzY5ODI1NzkyNzI5Mzc1NzI8L0NJRD4NCjwvVGlja2V0Pg==&lt;/Data&gt;&lt;Sig&gt;SLE8LXFmBW/2nMY9t+lG/7w4APZt3Z5U4nsu3G7KSWSdTEvTt9mt2kdssQaxxjEhy8udrLlC2dFSQXtHI/6mmbHhtaf7wx2WvRb4F1ayv5kZmrp5lJPkEXhdSwzJHlYPZM530Gsr7Md9MW4w67F7ct7i2MhsQyBLXr5nEDLlILHjTNUkbIa31IZJ5Qpwnr7Cj4XLPYOl8Phl6mHSjWdLo/CvohxRnAb/akABRyIhdd4rIvZREYsYhjSyZ/RLc6j0eLF7zkn8jjLKVGkIIFNvcGGnv/9ZtQ4zO5a/OkNB18Pvj6excNHt8zeCXiPomIikZrUOEZ4sshYRAJ7/5k/PAA==&lt;/Sig&gt;&lt;/SignedTicket&gt;") };

            let result = AbfindContactsPagedResultType{ groups: Some(groups), contacts: Some(array_of_contact), circle_result: Some(circle_result), ab: ab };
            let body_body = AbfindContactsPagedResponse{ ab_find_contacts_paged_result: result };
            let body = SoapAbfindContactsPagedResponseMessage{ body: body_body, fault: None };
            let r = AbfindContactsPagedResponseMessageSoapEnvelope{header: Some(service_header_container), body };

            let serialized = to_string(&r).unwrap();
            println!("{}", serialized);
        }
    }

    use chrono::Local;
    use yaserde::ser::to_string;
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::shared::models::role_list::RoleList;
    use crate::shared::models::uuid::Uuid;
    use crate::soap::abch::msnab_datatypes::{AbInfoType, ArrayOfContactType, CircleResultType, ContactType, GroupInfoType, GroupType, Annotation, ArrayOfAnnotation, AbType, AddressBookType, Circles, CircleInverseInfoType, ContentType, ContentHandleType, ContentInfoType, PersonalInfoType, MembershipInfoType, CirclePersonalMembershipType, RelationshipState, RoleId};
    use crate::soap::abch::msnab_faults::SoapFault;

    use crate::soap::abch::service_header::ServiceHeaderContainer;
    use crate::soap::error::SoapMarshallError;
    use crate::soap::traits::xml::ToXml;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAbfindContactsPagedResponseMessage {
        #[yaserde(rename = "ABFindContactsPagedResponse", default)]
        pub body: AbfindContactsPagedResponse,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "ABFindContactsPagedResponse",
    namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    prefix = "nsi1"
    default_namespace="nsi1"
    )]
    pub struct AbfindContactsPagedResponse {
        #[yaserde(rename = "ABFindContactsPagedResult", prefix="nsi1")]
        pub ab_find_contacts_paged_result: AbfindContactsPagedResultType,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "groups",
    namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    prefix = "nsi1"
    default_namespace="nsi1")]
    pub struct Groups {
        #[yaserde(rename = "Group", prefix="nsi1")]
        pub group: Vec<GroupType>,
    }


    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABFindContactsPagedResult", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    prefix = "nsi1",
    default_namespace="nsi1")]
    pub struct AbfindContactsPagedResultType {
        #[yaserde(rename = "Groups", prefix="nsi1")]
        pub groups: Option<Groups>,
        #[yaserde(rename = "Contacts", prefix="nsi1")]
        pub contacts: Option<ArrayOfContactType>,
        #[yaserde(rename = "CircleResult", prefix="nsi1")]
        pub circle_result: Option<CircleResultType>,
        #[yaserde(rename = "Ab", prefix="nsi1")]
        pub ab: Ab,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ab")]
    pub struct Ab {
        #[yaserde(rename = "abId", default)]
        pub ab_id: String,
        #[yaserde(rename = "abInfo", default)]
        pub ab_info: AbInfoType,
        #[yaserde(rename = "lastChange", default)]
        pub last_change: String,
        #[yaserde(rename = "DynamicItemLastChanged", default)]
        pub dynamic_item_last_changed: String,
        #[yaserde(rename = "RecentActivityItemLastChanged", default)]
        pub recent_activity_item_last_changed: Option<String>,
        #[yaserde(rename = "createDate", default)]
        pub create_date: String,
        #[yaserde(rename = "propertiesChanged", default)]
        pub properties_changed: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soap"
    )]
    pub struct AbfindContactsPagedResponseMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<ServiceHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAbfindContactsPagedResponseMessage,
    }

    impl ToXml for AbfindContactsPagedResponseMessageSoapEnvelope {
        type Error = SoapMarshallError;

        fn to_xml(&self) -> Result<String, Self::Error>  {
            to_string(self).map_err(|e| SoapMarshallError::SerializationError { message: e})
        }

    }

    impl AbfindContactsPagedResponseMessageSoapEnvelope {

        pub fn new_circle(ab_id: &str, cache_key: &str, mut contacts: Vec<ContactType>) -> Self {
            let room_id = ab_id.trim_start_matches("00000000-0000-0000-0009-");
            let uuid = Uuid::from_seed(room_id);

            let msn_addr = format!("{}@hotmail.com", ab_id);
            let me = ContactType::new_me_circle(&uuid, &msn_addr, ab_id);

            contacts.push(me);

            let now = Local::now();

            let create_date = String::from("2014-10-31T00:00:00Z");

            let ab_info_type = AbInfoType{ migrated_to: None, beta_status: None, name: None, owner_puid: 0, owner_cid: uuid.to_decimal_cid(), owner_email:Some(msn_addr), f_default: true, joined_namespace: false, is_bot: false, is_parent_managed: false, account_tier: None, account_tier_last_changed: String::from("0001-01-01T00:00:00"), profile_version: 0, subscribe_external_partner: false, notify_external_partner: false, address_book_type: AddressBookType::Group, messenger_application_service_created: None, is_beta_migrated: None, last_relevance_update: None };
            let ab = Ab{ ab_id: ab_id.to_string(), ab_info: ab_info_type, last_change: now.format("%Y-%m-%dT%H:%M:%SZ").to_string(), dynamic_item_last_changed: String::from("0001-01-01T00:00:00"), recent_activity_item_last_changed: None, create_date: create_date.clone(), properties_changed: String::new() };

            let array_of_contact = ArrayOfContactType{ contact: contacts };
            let result = AbfindContactsPagedResultType{ groups: None, contacts: Some(array_of_contact), circle_result: None, ab };
            let body_body = AbfindContactsPagedResponse{ ab_find_contacts_paged_result: result };
            let body = SoapAbfindContactsPagedResponseMessage{ body: body_body, fault: None };

            Self{header: Some(ServiceHeaderContainer::new(cache_key)), body }
        }

        pub fn new_individual(uuid: Uuid, cache_key: &str, msn_addr: &str, display_name: &str, mut contacts: Vec<ContactType>, profile_update: bool, circles: Vec<ContactType>) -> Self {
            contacts.extend_from_slice(&circles);

            let now = Local::now();

            let create_date = String::from("2014-10-31T00:00:00Z");

            let ab_info_type = AbInfoType{ migrated_to: None, beta_status: None, name: None, owner_puid: 0, owner_cid: uuid.to_decimal_cid(), owner_email:Some(msn_addr.to_string()), f_default: true, joined_namespace: false, is_bot: false, is_parent_managed: false, account_tier: None, account_tier_last_changed: String::from("0001-01-01T00:00:00"), profile_version: 0, subscribe_external_partner: false, notify_external_partner: false, address_book_type: AddressBookType::Individual, messenger_application_service_created: None, is_beta_migrated: None, last_relevance_update: None };
            let ab = Ab{ ab_id: Uuid::nil().to_string(), ab_info: ab_info_type, last_change: now.format("%Y-%m-%dT%H:%M:%SZ").to_string(), dynamic_item_last_changed: String::from("0001-01-01T00:00:00"), recent_activity_item_last_changed: None, create_date: create_date.clone(), properties_changed: String::new() };

            let mut contact_array = contacts;
            contact_array.push(ContactType::new_me(&uuid, &msn_addr, &display_name, profile_update));
            let array_of_contact = ArrayOfContactType{ contact: contact_array };

            let mut favorite_annotation_arary : Vec<Annotation> = Vec::new();
            favorite_annotation_arary.push(Annotation::new_display(Some(true)));
            let favorite_array_of_annotations = ArrayOfAnnotation{ annotation: favorite_annotation_arary };


            let favorite_group = GroupType{ group_id: String::from("1ae28c79-c963-4fe6-8339-d72a0f7c8bd2"), group_info: GroupInfoType{ annotations: Some(favorite_array_of_annotations), group_type: Some(String::from("c8529ce2-6ead-434d-881f-341e17db3ff8")), name: Some(String::from("Favorites")), is_not_mobile_visible: Some(false), is_private: Some(false), is_favorite: Some(true), f_messenger: None }, properties_changed: String::new(), f_deleted: Some(false), last_change: Some(create_date.clone())};
            let mut group_array : Vec<GroupType> = Vec::new();
            group_array.push(favorite_group);
            let groups = Groups{ group: group_array };

            let circles = {
              if circles.is_empty() {
                  None
              } else {

                  let mut circle_inverse_info = Vec::with_capacity(circles.len());

                  for circle in circles {
                      let display_name = circle.contact_info.expect("to be here").display_name.unwrap();

                      let current = CircleInverseInfoType {
                          content: ContentType {
                              handle: ContentHandleType {
                                  id: circle.contact_id.expect("a contact id to be present")
                              },
                              info: ContentInfoType {
                                  domain: 1,
                                  hosted_domain: "live.com".to_string(),
                                  rs_type: 2,
                                  membership_access: 0,
                                  is_presence_enabled: false,
                                  request_membership_option: 2,
                                  display_name: display_name.clone(),
                                  profile_last_updated: Some("0001-01-01T00:00:00".into()),
                                  changes: None,
                                  create_date: Some("0001-01-01T00:00:00".into()),
                                  last_changed: Some(now.format("%Y-%m-%dT%H:%M:%SZ").to_string()),
                              }
                          },
                          personal_info: PersonalInfoType {
                              membership_info: MembershipInfoType {
                                  circle_personal_membership: CirclePersonalMembershipType {
                                      role: RoleId::StateAccepted,
                                      state: "Accepted".to_string(), //RelationshipState
                                  },
                              },
                              name: display_name,
                              is_not_mobile_visible: false,
                              is_favorite: false,
                              is_family: false,
                              changes: None,
                              notes: None,
                          },
                          deleted: false,
                      };

                      circle_inverse_info.push(current);
                  }

                  let circles = Circles {
                      circle_inverse_info,
                  };

                  Some(circles)
              }
            };

            let circle_result = CircleResultType{ circles, circle_ticket: String::from("&lt;?xml version=\"1.0\" encoding=\"utf-16\"?&gt;&lt;SignedTicket xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\" ver=\"1\" keyVer=\"1\"&gt;&lt;Data&gt;PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0idXRmLTE2Ij8+DQo8VGlja2V0IHhtbG5zOnhzaT0iaHR0cDovL3d3dy53My5vcmcvMjAwMS9YTUxTY2hlbWEtaW5zdGFuY2UiIHhtbG5zOnhzZD0iaHR0cDovL3d3dy53My5vcmcvMjAwMS9YTUxTY2hlbWEiPg0KICA8VFM+MDAwMC0wMS0wMVQwMDowMDowMDwvVFM+DQogIDxDSUQ+LTc3NzY5ODI1NzkyNzI5Mzc1NzI8L0NJRD4NCjwvVGlja2V0Pg==&lt;/Data&gt;&lt;Sig&gt;SLE8LXFmBW/2nMY9t+lG/7w4APZt3Z5U4nsu3G7KSWSdTEvTt9mt2kdssQaxxjEhy8udrLlC2dFSQXtHI/6mmbHhtaf7wx2WvRb4F1ayv5kZmrp5lJPkEXhdSwzJHlYPZM530Gsr7Md9MW4w67F7ct7i2MhsQyBLXr5nEDLlILHjTNUkbIa31IZJ5Qpwnr7Cj4XLPYOl8Phl6mHSjWdLo/CvohxRnAb/akABRyIhdd4rIvZREYsYhjSyZ/RLc6j0eLF7zkn8jjLKVGkIIFNvcGGnv/9ZtQ4zO5a/OkNB18Pvj6excNHt8zeCXiPomIikZrUOEZ4sshYRAJ7/5k/PAA==&lt;/Sig&gt;&lt;/SignedTicket&gt;") };

            let result = AbfindContactsPagedResultType{ groups: Some(groups), contacts: Some(array_of_contact), circle_result: Some(circle_result), ab };
            let body_body = AbfindContactsPagedResponse{ ab_find_contacts_paged_result: result };
            let body = SoapAbfindContactsPagedResponseMessage{ body: body_body, fault: None };

            Self{header: Some(ServiceHeaderContainer::new(cache_key)), body }
        }
    }

}