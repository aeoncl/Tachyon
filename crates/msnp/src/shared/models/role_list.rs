use strum_macros::{Display, EnumIter};
use yaserde_derive::{YaDeserialize, YaSerialize};

#[derive(Debug, Display, YaSerialize, YaDeserialize, Clone, PartialEq, Eq, Hash, EnumIter)]
#[yaserde(
	rename = "RoleId",namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub enum RoleList {
	//Contact doesn't belong to the contact list, but belongs to the address book
	None = 0x00,
	//Contact belongs to our contact list
	Forward = 0x01,
	//Contact is always explicitely allowed to see our presence
	Allow = 0x02,
	// Contact is always explicitely forbidden from seeing our presence
	Block = 0x04,
	// We belong to the FORWARD list of the contact
	Reverse = 0x08,
	// Contact pending
	Pending = 0x10
}

impl Default for RoleList {
    fn default() -> Self {
        RoleList::Allow
    }
}