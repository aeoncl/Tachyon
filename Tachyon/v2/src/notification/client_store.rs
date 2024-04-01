use matrix_sdk::Client;
use msnp::msnp::notification::command::usr::TicketToken;
use msnp::msnp::notification::models::endpoint_guid::EndpointGuid;
use tokio::sync::oneshot;

pub struct ClientStore {
    pub email_addr: Option<String>,
    pub ticket_token: Option<TicketToken>,
    pub endpoint_guid: Option<EndpointGuid>,
    pub matrix_client: Option<Client>
}

impl Default for ClientStore {
    fn default() -> Self {
        Self {
            email_addr: None,
            ticket_token: None,
            endpoint_guid: None,
            matrix_client: None,
        }
    }
}

pub enum ClientStoreEvent{
    Setter(ClientStoreSetterEvent),
    Getter(ClientStoreGetterEvent)
}

pub enum ClientStoreSetterEvent {
    SetClientEmail(String),
    SetTicketTokenAndEndpoint(TicketToken, EndpointGuid),
    SetMatrixClient(Client)
}

pub enum ClientStoreGetterEvent {
    GetClientEmail(oneshot::Sender<Option<String>>),
    GetClientTicketToken(oneshot::Sender<Option<TicketToken>>),
    GetClientEndpointGuid(oneshot::Sender<Option<EndpointGuid>>),
    GetMatrixClient(oneshot::Sender<Option<Client>>)
}



