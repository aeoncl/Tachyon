use std::collections::{HashMap, VecDeque};
use log::info;
use matrix_sdk::Client;
use matrix_sdk::ruma::{OwnedRoomId, OwnedUserId};
use msnp::msnp::models::contact_list::ContactList;
use msnp::msnp::notification::models::endpoint_guid::EndpointGuid;
use msnp::msnp::switchboard::command::command::SwitchboardServerCommand;
use msnp::shared::models::ticket_token::TicketToken;
use msnp::soap::error::SoapMarshallError;
use thiserror::Error;
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::sync::{mpsc, oneshot};
use tokio::sync::mpsc::error::SendError;
use tokio::sync::oneshot::error::RecvError;

pub struct ClientData {
    pub email_addr: Option<String>,
    pub ticket_token: Option<TicketToken>,
    pub endpoint_guid: Option<EndpointGuid>,
    pub matrix_client: Option<Client>,
    pub contact_list: ContactList,
    pub switchboards: HashMap<OwnedRoomId, SwitchboardHandle>,
    pub soap_holder: SoapHolder
}

impl Default for ClientData {
    fn default() -> Self {
        Self {
            email_addr: None,
            ticket_token: None,
            endpoint_guid: None,
            matrix_client: None,
            contact_list: Default::default(),
            switchboards: Default::default(),
            soap_holder: Default::default(),
        }
    }
}

pub struct ClientStoreOperation {
    pub client_key: String,
    pub operation: ClientDataOperation
}

impl ClientStoreOperation {
    pub fn getter(client_key: String, operation: ClientDataGetter) -> Self {
        Self {
            client_key,
            operation: ClientDataOperation::Getter(operation),
        }
    }

    pub fn setter(client_key: String, operation: ClientDataSetter) -> Self {
        Self {
            client_key,
            operation: ClientDataOperation::Setter(operation),
        }
    }

    pub fn drop(client_key: String) -> Self {
        Self {
            client_key,
            operation: ClientDataOperation::Drop(),
        }
    }
}

pub enum ClientDataOperation {
    Setter(ClientDataSetter),
    Getter(ClientDataGetter),
    Drop()
}

pub enum ClientDataSetter {
    SetClientEmail(String),
    SetTicketTokenAndEndpoint(TicketToken, EndpointGuid),
    SetMatrixClient(Client),
    SetSwitchboard(OwnedRoomId, Option<SwitchboardHandle>)
}

pub enum ClientDataGetter {
    GetClientEmail(oneshot::Sender<Option<String>>),
    GetClientTicketToken(oneshot::Sender<Option<TicketToken>>),
    GetClientEndpointGuid(oneshot::Sender<Option<EndpointGuid>>),
    GetMatrixClient(oneshot::Sender<Option<Client>>),
    GetSwitchboard(OwnedRoomId, oneshot::Sender<Option<SwitchboardHandle>>)
}

#[derive(Clone)]
pub struct ClientStoreFacade {
    sender: Sender<ClientStoreOperation>
}

#[derive(Error, Debug)]
pub enum ClientStoreError {
    #[error(transparent)]
    SendError(#[from] SendError<ClientStoreOperation>),
    #[error(transparent)]
    RecvError(#[from] RecvError)
}

impl ClientStoreFacade {

    pub fn new(sender: Sender<ClientStoreOperation>) -> Self {
        Self{ sender }
    }
    pub async fn get_client_email(&self, client_key: &str) -> Result<Option<String>, ClientStoreError>{
        let (send, recv) = oneshot::channel::<Option<String>>();
        self.sender.send(ClientStoreOperation::getter(client_key.to_string(), ClientDataGetter::GetClientEmail(send))).await?;
        Ok(recv.await?)
    }

    pub async fn set_client_email(&self, client_key: &str, email_addr: String) -> Result<(), ClientStoreError> {
        self.sender.send(ClientStoreOperation::setter(client_key.to_string(), ClientDataSetter::SetClientEmail(email_addr))).await?;
        Ok(())
    }

    pub async fn set_ticket_token_and_endpoint_guid(&self, client_key: &str, ticket_token: TicketToken, endpoint_guid: EndpointGuid) -> Result<(), ClientStoreError> {
        self.sender.send(ClientStoreOperation::setter(client_key.to_string(), ClientDataSetter::SetTicketTokenAndEndpoint(ticket_token, endpoint_guid))).await?;
        Ok(())
    }

    pub async fn get_ticket_token(&self, client_key: &str) ->  Result<Option<TicketToken>, ClientStoreError> {
        let (send, recv) = oneshot::channel::<Option<TicketToken>>();
        self.sender.send(ClientStoreOperation::getter(client_key.to_string(), ClientDataGetter::GetClientTicketToken(send))).await?;
        Ok(recv.await?)
    }

    pub async fn get_endpoint_guid(&self, client_key: &str) ->  Result<Option<EndpointGuid>, ClientStoreError> {
        let (send, recv) = oneshot::channel::<Option<EndpointGuid>>();
        self.sender.send(ClientStoreOperation::getter(client_key.to_string(), ClientDataGetter::GetClientEndpointGuid(send))).await?;
        Ok(recv.await?)
    }

    pub async fn set_matrix_client(&self, client_key: &str, matrix_client: Client) -> Result<(), ClientStoreError> {
        self.sender.send(ClientStoreOperation::setter(client_key.to_string(), ClientDataSetter::SetMatrixClient(matrix_client))).await?;
        Ok(())
    }

    pub async fn get_matrix_client(&self, client_key: &str) -> Result<Option<Client>, ClientStoreError>  {
        let (send, recv) = oneshot::channel::<Option<Client>>();
        self.sender.send(ClientStoreOperation::getter(client_key.to_string(), ClientDataGetter::GetMatrixClient(send))).await?;
        Ok(recv.await?)
    }

    pub async fn set_switchboard(&self, client_key: &str, id: OwnedRoomId, switchboard: Option<SwitchboardHandle>) -> Result<(), ClientStoreError> {
        self.sender.send(ClientStoreOperation::setter(client_key.to_string(), ClientDataSetter::SetSwitchboard(id, switchboard))).await?;
        Ok(())
    }

    pub async fn get_switchboard(&self, client_key: &str, id: OwnedRoomId ) -> Result<Option<SwitchboardHandle>, ClientStoreError> {
        let (send, recv) = oneshot::channel::<Option<SwitchboardHandle>>();
        self.sender.send(ClientStoreOperation::getter(client_key.to_string(), ClientDataGetter::GetSwitchboard(id, send))).await?;
        Ok(recv.await?)
    }

    pub async fn drop_client(&self, client_key: &str) -> Result<(), ClientStoreError>{
        self.sender.send(ClientStoreOperation::drop(client_key.to_string())).await?;
        Ok(())
    }

}

#[derive(Clone)]
pub struct SwitchboardHandle {
    room_id: String,
    msnp_sender: mpsc::Sender<SwitchboardServerCommand>,
    p2p_sender: mpsc::Sender<String>
}

pub struct SoapHolder {
    oims: VecDeque<String>,
    contacts: VecDeque<String>,
    memberships: VecDeque<String>
}

impl Default for SoapHolder {
    fn default() -> Self {
        Self{
            oims: Default::default(),
            contacts: Default::default(),
            memberships: Default::default(),
        }
    }
}



pub fn start_client_store_task(mut kill_recv: Receiver<()>) -> Sender<ClientStoreOperation> {
    let (sender, mut receiver) = mpsc::channel::<ClientStoreOperation>(300);

    let _result = tokio::spawn(async move {

        let mut client_store: HashMap<String, ClientData> = HashMap::new();

        loop {
            tokio::select! {
                store_operation = receiver.recv() => {
                    if let Some(store_operation) = store_operation {

                        if let ClientDataOperation::Drop() = store_operation.operation {
                            client_store.remove(&store_operation.client_key);
                            continue;
                        }

                        let client_data = client_store.entry(store_operation.client_key).or_default();

                        match store_operation.operation {
                            ClientDataOperation::Setter(setter) => {
                                match setter {
                                    ClientDataSetter::SetClientEmail(email_addr) => {
                                        client_data.email_addr = Some(email_addr);
                                    },
                                    ClientDataSetter::SetTicketTokenAndEndpoint(ticket_token, endpoint_guid) => {
                                        client_data.ticket_token = Some(ticket_token);
                                        client_data.endpoint_guid = Some(endpoint_guid);
                                    }
                                    ClientDataSetter::SetMatrixClient(client) => {
                                        client_data.matrix_client = Some(client);
                                    }
                                ClientDataSetter::SetSwitchboard(id,switchboard) => {
                                    match switchboard {
                                        None => {
                                                let _removed = client_data.switchboards.remove(&id);
                                            },
                                        Some(switchboard) => {
                                                let _previous = client_data.switchboards.insert(id, switchboard);
                                            }
                                        }

                                }}
                            },
                            ClientDataOperation::Getter(getter) => {
                                match getter {
                                    ClientDataGetter::GetClientEmail(channel) => {
                                        let _result = channel.send(client_data.email_addr.clone());
                                    },
                                    ClientDataGetter::GetClientTicketToken(channel) => {
                                        let _result = channel.send(client_data.ticket_token.clone());

                                    },
                                    ClientDataGetter::GetClientEndpointGuid(channel) => {
                                       let _result = channel.send(client_data.endpoint_guid.clone());
                                    }
                                    ClientDataGetter::GetMatrixClient(channel) => {
                                       let _result = channel.send(client_data.matrix_client.clone());
                                    }
                                    ClientDataGetter::GetSwitchboard(id,channel) => {
                                        let _result = channel.send(client_data.switchboards.get(&id).cloned());
                                    }
                                }

                            }
                            ClientDataOperation::Drop() => {}
                        }
                    }
                },
                _kill_signal = kill_recv.recv() => {
                    break;
                }
            }
        }
        info!("ClientStore task gracefully shutdown...");
    } );
    sender
}



