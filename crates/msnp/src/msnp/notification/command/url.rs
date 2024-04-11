
pub struct UrlClient {

    tr_id: u128,
    url_type: UrlType

}

pub enum UrlType {
    Inbox,
    Folders,
    Compose {
        email_addr: String
    },
    Profile {
        locale_id: String
    },
    Person {
        locale_id: String
    },
    Chat {
        locale_id: String
    },
    Mobile,
    AddrBook,
    AdvSearch,
    IntSearch

}