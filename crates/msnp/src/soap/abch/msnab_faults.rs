use yaserde::ser::to_string;
use yaserde_derive::{YaDeserialize, YaSerialize};

use crate::shared::models::uuid::Uuid;
use crate::soap::error::SoapMarshallError;
use crate::soap::traits::xml::ToXml;

#[derive(Debug, Default, YaSerialize, YaDeserialize)]
#[yaserde(
rename = "Envelope",
namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance"
namespace = "xsd: http://www.w3.org/2001/XMLSchema"
prefix = "soap"
)]
pub struct SoapFaultResponseEnvelope {
	#[yaserde(rename = "Body", prefix = "soap")]
	body: SoapFaultBody
}

impl ToXml for SoapFaultResponseEnvelope {
	type Error = SoapMarshallError;

	fn to_xml(&self) -> Result<String, Self::Error>  {
		to_string(self).map_err(|e| SoapMarshallError::SerializationError { message: e})
	}
}

impl SoapFaultResponseEnvelope {

	pub fn new_fullsync_required(soap_action: &str) -> Self {

		let additional_details = FaultAdditionalDetails{
			original_exception_error_message: Some("Full sync required.  Details: Delta syncs disabled.".into()),
			conflict_object_id: None
		};

		let fault_detail = FaultDetail{
			error_code: Some(FaultErrorCode::FullSyncRequired),
			error_string: Some("Full sync required.  Details: Delta syncs disabled.".into()),
			machine_name: Some("TACHEPSILON3".into()),
			parameter_fault: None,
			additional_details: Some(additional_details),
		};

		let soap_fault = SoapFault{
			fault_code: Some("soap:Client".into()),
			fault_string: Some("Full sync required.  Details: Delta syncs disabled.".into()),
			fault_actor: Some(soap_action.to_owned()),
			detail: Some(fault_detail),
		};

		SoapFaultResponseEnvelope {
			body: SoapFaultBody {
				fault: soap_fault,
			}
		}
	}

	pub fn new_contact_already_exists(soap_action: String, confict_object_id: &Uuid) -> Self {

		let additional_details = FaultAdditionalDetails{
			original_exception_error_message: None,
			conflict_object_id: Some(confict_object_id.to_string())
		};


		let fault_detail = FaultDetail{
			error_code: Some(FaultErrorCode::ContactAlreadyExists),
			error_string: Some("Contact Already Exists".into()),
			machine_name: Some("TACHEPSILON3".into()),
			parameter_fault: None,
			additional_details: Some(additional_details),
		};

		let soap_fault = SoapFault{
			fault_code: Some("soap:Client".into()),
			fault_string: Some("Contact Already Exists".into()),
			fault_actor: Some(soap_action),
			detail: Some(fault_detail),
		};

		SoapFaultResponseEnvelope {
			body: SoapFaultBody {
				fault: soap_fault,
			}
		}
	}

	pub fn new_contact_doesnt_exist(soap_action: String, confict_object_id: &Uuid) -> Self {

		let additional_details = FaultAdditionalDetails{
			original_exception_error_message: None,
			conflict_object_id: Some(confict_object_id.to_string())
		};


		let fault_detail = FaultDetail{
			error_code: Some(FaultErrorCode::ContactDoesNotExist),
			error_string: Some("Contact Does Not Exist".into()),
			machine_name: Some("TACHEPSILON3".into()),
			parameter_fault: None,
			additional_details: Some(additional_details),
		};

		let soap_fault = SoapFault{
			fault_code: Some("soap:Client".into()),
			fault_string: Some("Contact Does Not Exist".into()),
			fault_actor: Some(soap_action),
			detail: Some(fault_detail),
		};

		SoapFaultResponseEnvelope {
			body: SoapFaultBody {
				fault: soap_fault,
			}
		}

	}

	pub fn new_email_missing_at_sign(soap_action: String) -> Self {
		let additional_details = FaultAdditionalDetails{
			original_exception_error_message: Some("Malformed email Argument Email missing '@' character".into()),
			conflict_object_id: None
		};

		let fault_detail = FaultDetail{
			error_code: Some(FaultErrorCode::BadEmailArgument),
			error_string: Some("Malformed email Argument Email missing '@' character".into()),
			machine_name: Some("TACHEPSILON3".into()),
			parameter_fault: None,
			additional_details: Some(additional_details),
		};

		let soap_fault = SoapFault{
			fault_code: Some("soap:Client".into()),
			fault_string: Some("Malformed email Argument Email missing '@' character".into()),
			fault_actor: Some(soap_action),
			detail: Some(fault_detail),
		};

		SoapFaultResponseEnvelope {
			body: SoapFaultBody {
				fault: soap_fault,
			}
		}

	}


	pub fn new_email_missing_dot(soap_action: String) -> Self {
		let additional_details = FaultAdditionalDetails{
			original_exception_error_message: Some("Malformed email Argument Email missing '.' character".into()),
			conflict_object_id: None
		};

		let fault_detail = FaultDetail{
			error_code: Some(FaultErrorCode::BadEmailArgument),
			error_string: Some("Malformed email Argument Email missing '.' character".into()),
			machine_name: Some("TACHEPSILON3".into()),
			parameter_fault: None,
			additional_details: Some(additional_details),
		};

		let soap_fault = SoapFault{
			fault_code: Some("soap:Client".into()),
			fault_string: Some("Malformed email Argument Email missing '.' character".into()),
			fault_actor: Some(soap_action),
			detail: Some(fault_detail),
		};

		SoapFaultResponseEnvelope {
			body: SoapFaultBody {
				fault: soap_fault,
			}
		}

	}

	pub fn new_group_already_exists(soap_action: String, confict_object_id: &Uuid) -> Self {

		let additional_details = FaultAdditionalDetails{
			original_exception_error_message: None,
			conflict_object_id: Some(confict_object_id.to_string())
		};


		let fault_detail = FaultDetail{
			error_code: Some(FaultErrorCode::GroupAlreadyExists),
			error_string: Some("Group Already Exists".into()),
			machine_name: Some("TACHEPSILON3".into()),
			parameter_fault: None,
			additional_details: Some(additional_details),
		};

		let soap_fault = SoapFault{
			fault_code: Some("soap:Client".into()),
			fault_string: Some("Group Already Exists".into()),
			fault_actor: Some(soap_action),
			detail: Some(fault_detail),
		};

		SoapFaultResponseEnvelope {
			body: SoapFaultBody {
				fault: soap_fault,
			}
		}

	}

	pub fn new_group_name_too_long(soap_action: String) -> Self {
		let additional_details = FaultAdditionalDetails{
			original_exception_error_message: Some("Argument Exceeded Allowed Length GroupName exceeded length".into()),
			conflict_object_id: None
		};

		let fault_detail = FaultDetail{
			error_code: Some(FaultErrorCode::BadArgumentLength),
			error_string: Some("Argument Exceeded Allowed Length GroupName exceeded length".into()),
			machine_name: Some("TACHEPSILON3".into()),
			parameter_fault: Some("GroupName".into()),
			additional_details: Some(additional_details),
		};

		let soap_fault = SoapFault{
			fault_code: Some("soap:Client".into()),
			fault_string: Some("Argument Exceeded Allowed Length GroupName exceeded length".into()),
			fault_actor: Some(soap_action),
			detail: Some(fault_detail),
		};

		SoapFaultResponseEnvelope {
			body: SoapFaultBody {
				fault: soap_fault,
			}
		}

	}

	pub fn new_invalid_passport_user(soap_action: String, msn_addr: &str) -> Self {
		let additional_details = FaultAdditionalDetails{
			original_exception_error_message: Some(format!("The Passport user specified is invalid SignInName: {}", msn_addr)),
			conflict_object_id: None
		};

		let fault_detail = FaultDetail{
			error_code: Some(FaultErrorCode::InvalidPassportUser),
			error_string: Some(format!("The Passport user specified is invalid SignInName: {}", msn_addr)),
			machine_name: Some("TACHEPSILON3".into()),
			parameter_fault: Some("GroupName".into()),
			additional_details: Some(additional_details),
		};

		let soap_fault = SoapFault{
			fault_code: Some("soap:Client".into()),
			fault_string: Some(format!("The Passport user specified is invalid SignInName: {}", msn_addr)),
			fault_actor: Some(soap_action),
			detail: Some(fault_detail),
		};

		SoapFaultResponseEnvelope {
			body: SoapFaultBody {
				fault: soap_fault,
			}
		}

	}

	pub fn new_member_does_not_exist(soap_action: String) -> Self {
		let additional_details = FaultAdditionalDetails{
			original_exception_error_message: None,
			conflict_object_id: None
		};

		let fault_detail = FaultDetail{
			error_code: Some(FaultErrorCode::MemberDoesNotExist),
			error_string: Some("Member does not exist".into()),
			machine_name: Some("TACHEPSILON3".into()),
			parameter_fault: Some("GroupName".into()),
			additional_details: Some(additional_details),
		};

		let soap_fault = SoapFault{
			fault_code: Some("soap:Client".into()),
			fault_string: Some("Member does not exist".into()),
			fault_actor: Some(soap_action),
			detail: Some(fault_detail),
		};
		SoapFaultResponseEnvelope {
			body: SoapFaultBody {
				fault: soap_fault,
			}
		}

	}

	pub fn new_user_does_not_exist(soap_action: String) -> Self {
		let additional_details = FaultAdditionalDetails{
			original_exception_error_message: None,
			conflict_object_id: None
		};

		let fault_detail = FaultDetail{
			error_code: Some(FaultErrorCode::UserDoesNotExist),
			error_string: Some("User does not exist".into()),
			machine_name: Some("TACHEPSILON3".into()),
			parameter_fault: Some("GroupName".into()),
			additional_details: Some(additional_details),
		};

		let soap_fault = SoapFault{
			fault_code: Some("soap:Client".into()),
			fault_string: Some("User does not exist".into()),
			fault_actor: Some(soap_action),
			detail: Some(fault_detail),
		};

		SoapFaultResponseEnvelope {
			body: SoapFaultBody {
				fault: soap_fault,
			}
		}

	}

	pub fn new_generic(fault_string: String) -> Self {
		let soap_fault = SoapFault{
			fault_code: Some("soap:Client".into()),
			fault_string: Some(fault_string),
			fault_actor: None,
			detail: None,
		};

		SoapFaultResponseEnvelope {
			body: SoapFaultBody {
				fault: soap_fault,
			}
		}
	}

	pub fn new_unknown_soap_action(soap_action: String) -> Self {

		let soap_fault = SoapFault{
			fault_code: Some("SOAP-ENV:Server".into()),
			fault_string: Some(format!("Function '{}' doesn't exist", soap_action)),
			fault_actor: None,
			detail: None,
		};

		SoapFaultResponseEnvelope {
			body: SoapFaultBody {
				fault: soap_fault,
			}
		}
	}

	pub fn new_deprecated_soap_action(soap_action: String) -> Self {
		let additional_details = FaultAdditionalDetails{
			original_exception_error_message: Some(format!("API {} no longer supported", &soap_action)),
			conflict_object_id: None
		};

		let fault_detail = FaultDetail{
			error_code: Some(FaultErrorCode::Forbidden),
			error_string: Some(format!("API {} no longer supported", &soap_action)),
			machine_name: Some("TACHEPSILON3".into()),
			parameter_fault: Some("GroupName".into()),
			additional_details: Some(additional_details),
		};

		let soap_fault = SoapFault{
			fault_code: Some("soap:Client".into()),
			fault_string: Some(format!("API {} no longer supported", &soap_action)),
			fault_actor: Some(soap_action),
			detail: Some(fault_detail),
		};

		SoapFaultResponseEnvelope {
			body: SoapFaultBody {
				fault: soap_fault,
			}
		}
	}




}

#[derive(Debug, Default, YaSerialize, YaDeserialize)]
#[yaserde(
rename = "Body",
namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
prefix = "soap"
)]
pub struct SoapFaultBody {
	#[yaserde(rename = "Fault", prefix="soap")]
	fault: SoapFault
}

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "Fault",
	namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
	prefix = "soap",
)]
pub struct SoapFault {
	#[yaserde(rename = "faultcode", default)]
	pub fault_code: Option<String>,
	#[yaserde(rename = "faultstring", default)]
	pub fault_string: Option<String>,
	#[yaserde(rename = "faultactor", default)]
	pub fault_actor: Option<String>,
	#[yaserde(rename = "detail", default)]
	pub detail: Option<FaultDetail>
}

impl std::error::Error for SoapFault {}

impl std::fmt::Display for SoapFault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (&self.fault_code, &self.fault_string) {
            (None, None) => Ok(()),
            (None, Some(fault_string)) => f.write_str(fault_string),
            (Some(fault_code), None) => f.write_str(fault_code),
            (Some(fault_code), Some(fault_string)) => {
                f.write_str(fault_code)?;
                f.write_str(": ")?;
                f.write_str(fault_string)
            }
        }
    }
}

#[derive(Debug, YaSerialize, YaDeserialize, Clone)]
#[yaserde(rename = "FindMembership",
namespace = "nsi1: http://www.msn.com/webservices/AddressBook")]
pub struct FaultDetail{
	#[yaserde(rename = "errorcode", prefix="nsi1")]
	pub error_code: Option<FaultErrorCode>,
	#[yaserde(rename = "errorstring", prefix="nsi1")]
	pub error_string: Option<String>,
	#[yaserde(rename = "machineName", prefix="nsi1")]
	pub machine_name: Option<String>,
	#[yaserde(rename = "parameterFault", prefix="nsi1")]
	pub parameter_fault: Option<String>,
	#[yaserde(rename = "additionalDetails")]
	pub additional_details: Option<FaultAdditionalDetails>

}

#[derive(Debug, YaSerialize, YaDeserialize, Clone)]
pub enum FaultErrorCode {
	InvalidPassportUser,
	BadArgumentLength,
	BadEmailArgument,
	GroupAlreadyExists,
	FullSyncRequired,
	ContactDoesNotExist,
	ContactAlreadyExists,
	Forbidden,
	MemberDoesNotExist,
	UserDoesNotExist,
	Unknown
}

impl Default for FaultErrorCode{
	fn default() -> Self {
		FaultErrorCode::Unknown
	}
}

#[derive(Debug, YaSerialize, YaDeserialize, Clone)]
pub struct FaultAdditionalDetails {
	#[yaserde(rename = "originalExceptionErrorMessage")]
	pub original_exception_error_message: Option<String>,
	#[yaserde(rename = "conflictObjectId")]
	pub conflict_object_id: Option<String>,

}


#[cfg(test)]
mod tests {
	use yaserde::de::from_str;
	use yaserde::ser::to_string;

	use crate::shared::models::uuid::Uuid;
	use crate::soap::abch::msnab_faults::SoapFaultResponseEnvelope;

	#[test]
	fn test_deserialize() {
		let req = r#"<?xml version="1.0" encoding="UTF-8"?>
							<soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/">
								<soap:Body>
									<soap:Fault>
										<faultcode>soap:Client</faultcode>
										<faultstring>Generic SOAP fault</faultstring>
									</soap:Fault>
								</soap:Body>
							</soap:Envelope>"#;

		let deser: SoapFaultResponseEnvelope = from_str(&req).expect("to work");
		assert_eq!("soap:Client", &deser.body.fault.fault_code.expect("to be here"));
	}

	#[test]
	fn test_serialize_fault_response() {

		let response = SoapFaultResponseEnvelope::new_contact_already_exists("http://www.msn.com/webservices/AddressBook/SoapAction".into(), &Uuid::from_seed("hey"));
		let response_serialized = to_string(&response).unwrap();

		println!("{}", response_serialized);

	}

}
