use yaserde_derive::{YaDeserialize, YaSerialize};

#[derive(Debug, Default, YaSerialize, Clone)]
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

#[derive(Debug, YaSerialize, Clone)]
pub struct FaultDetail{
	#[yaserde(rename = "errorcode", namespace="xmlns: http://www.msn.com/webservices/AddressBook")]
	pub error_code: Option<FaultErrorCode>,
	#[yaserde(rename = "errorstring", namespace="xmlns: http://www.msn.com/webservices/AddressBook")]
	pub error_string: Option<String>,
	#[yaserde(rename = "machineName", namespace="xmlns: http://www.msn.com/webservices/AddressBook")]
	pub machine_name: Option<String>,
	#[yaserde(rename = "parameterFault", namespace="xmlns: http://www.msn.com/webservices/AddressBook")]
	pub parameter_fault: Option<String>,
	#[yaserde(rename = "additionalDetails")]
	pub additional_details: Option<FaultAdditionalDetails>

}

#[derive(Debug, YaSerialize, Clone)]
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

#[derive(Debug, YaSerialize, Clone)]
pub struct FaultAdditionalDetails {
	#[yaserde(rename = "originalExceptionErrorMessage")]
	pub original_exception_error_message: Option<String>,
	#[yaserde(rename = "conflictObjectId")]
	pub conflict_object_id: Option<String>,

}

#[derive(Debug, YaSerialize, Clone)]
#[yaserde(
rename = "Envelope",
namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
namespace = "xsd: http://www.w3.org/2001/XMLSchema",
prefix = "soap",
)]
pub struct FaultResponse {

	#[yaserde(rename = "Body", prefix="soap")]
	pub body: FaultBody

}

#[derive(Debug, YaSerialize, Clone)]
pub struct FaultBody{
	#[yaserde(rename = "Fault", prefix="soap")]
	pub fault: SoapFault
}

pub mod factory {
	use crate::generated::msn_ab_faults::{FaultBody, FaultResponse, SoapFault};

	pub fn get_fault_response(soap_fault: SoapFault) -> FaultResponse {
		return FaultResponse{ body: FaultBody { fault: soap_fault} };
	}

	pub mod soap_fault {

		use crate::generated::msn_ab_faults::{FaultAdditionalDetails, FaultDetail, FaultErrorCode, SoapFault};
		use crate::models::uuid::UUID;


		pub fn get_fullsync_required(soap_action: String) -> SoapFault {

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
				fault_actor: Some(soap_action),
				detail: Some(fault_detail),
			};
			return soap_fault;
		}

		pub fn get_contact_already_exists(soap_action: String, confict_object_id: &UUID) -> SoapFault {

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
			return soap_fault;
		}

		pub fn get_contact_doesnt_exist(soap_action: String, confict_object_id: &UUID) -> SoapFault {

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
			return soap_fault;
		}

		pub fn get_email_missing_at_sign(soap_action: String) -> SoapFault {
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
			return soap_fault;
		}

		pub fn get_email_missing_dot(soap_action: String) -> SoapFault {
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
			return soap_fault;
		}

		pub fn get_group_already_exists(soap_action: String, confict_object_id: &UUID) -> SoapFault {

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
			return soap_fault;
		}

		pub fn get_group_name_too_long(soap_action: String) -> SoapFault {
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
			return soap_fault;
		}

		pub fn get_invalid_passport_user(soap_action: String, msn_addr: &str) -> SoapFault {
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
			return soap_fault;
		}

		pub fn get_member_does_not_exist(soap_action: String) -> SoapFault {
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
			return soap_fault;
		}

		pub fn get_user_does_not_exist(soap_action: String) -> SoapFault {
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
			return soap_fault;
		}
	}



}

#[cfg(test)]
mod tests {
	use crate::generated::msn_ab_faults;
	use yaserde::ser::to_string;
	use crate::models::uuid::UUID;

	#[test]
	fn test_serialize_fault_response() {

		let response = msn_ab_faults::factory::get_fault_response(msn_ab_faults::factory::soap_fault::get_contact_already_exists("http://www.msn.com/webservices/AddressBook/SoapAction".into(), &UUID::from_string("hey")));
		let response_serialized = to_string(&response).unwrap();

		println!("{}", response_serialized);

	}

}
