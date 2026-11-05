use crate::matrix::extensions::msn_user_resolver::{FindRoomFromEmail, ToMsnUser};
use crate::notification::models::client_out::ClientOut;
use crate::notification::models::session_state::SessionState;
use crate::tachyon::client::tachyon_client::TachyonClient;
use crate::tachyon::identifiers::is_sha1::IsSha1;
use log::warn;
use matrix_sdk::Client;
use msnp::msnp::notification::command::chg::ChgClient;
use msnp::msnp::notification::command::command::NotificationServerCommand;
use msnp::msnp::notification::command::iln::IlnServer;
use msnp::msnp::notification::command::ubx::{ExtendedPresenceContent, UbxPayload, UbxServer};
use msnp::msnp::notification::models::dialect::Dialect;
use msnp::msnp::notification::models::endpoint_data::EndpointData;
use msnp::shared::models::capabilities::ClientCapabilities;
use msnp::shared::models::display_name::DisplayName;
use msnp::shared::models::endpoint_id::EndpointId;
use msnp::shared::models::msn_object::MsnObject;
use msnp::shared::models::network_id_email::NetworkIdEmail;
use msnp::shared::models::presence_status::PresenceStatus;
use tokio::sync::mpsc::Sender;

/// Gather output for one forward-list contact: plain data only — every async
/// lookup (room, display name, avatar) is already resolved (§3.1, §10: the
/// gather stays per-flow, over only the fields this flow reads).
pub(crate) struct GatheredContact {
    pub(crate) network_id_email: NetworkIdEmail,
    pub(crate) display_name: Option<String>,
    pub(crate) avatar: Option<MsnObject>,
}

pub async fn handle_chg(command: ChgClient, local_store: &mut SessionState, client_data: TachyonClient, matrix_client: Client, command_sender: Sender<NotificationServerCommand>) -> Result<(), anyhow::Error>  {
    command_sender.send(NotificationServerCommand::CHG(command.clone())).await?;

    let tachyon_client = client_data.clone();

    if local_store.needs_initial_presence {
        local_store.needs_initial_presence = false;

        let dialect = local_store.dialect;

        // Spawn placement is shell policy, preserved exactly (§3.6): the fan-out
        // runs detached so the read loop keeps answering while the Matrix
        // lookups are in flight.
        tokio::spawn( async move {

            // Gather: async IO only.
            let contacts = {
                tachyon_client.get_contact_list().lock().unwrap().get_forward_list()
            };

            let mut gathered = Vec::new();

            for contact in contacts {

                if !contact.email_address.is_sha1_imprecise() {
                    continue;
                }

               let found_room =  matrix_client.find_room_from_email(&contact.email_address);

               let display_name = if let Ok(Some(room)) = &found_room {
                   if let Ok(msn_user) = room.to_msn_user_lazy().await {
                       msn_user.display_name
                   } else {
                       None
                   }
                    
                } else {
                    None
                };

                // A failed avatar lookup must not kill the fan-out — it used to
                // `.unwrap()` here and silently abort the whole spawned task
                // (§1.2). Log it, send the presence without avatar, carry on.
                let avatar = if let Ok(Some(room)) = &found_room {
                    match tachyon_client.get_avatar_as_msn_object(room.room_id()).await {
                        Ok(avatar) => avatar,
                        Err(error) => {
                            warn!("MSNP|NS: unable to build the avatar msnobj for {}: {}", contact.email_address.as_str(), error);
                            None
                        },
                    }
                } else { None };

                gathered.push(GatheredContact {
                    network_id_email: NetworkIdEmail {
                        network_id: contact.network_id.clone(),
                        email: contact.email_address.clone(),
                    },
                    display_name,
                    avatar,
                });
            }

            // Decide (pure), then render per dialect into the existing sink.
            for out in decide_chg(gathered, command.tr_id, dialect) {
                for server_command in render(out, command.tr_id, dialect) {
                    let _ = command_sender.send(server_command).await;
                }
            }
        });
        
        
    }



    //notif_sender.send(NotificationServerCommand::SDG())



    Ok(())
}

/// Pure decision step (§3.1): one `ContactPresence` per gathered contact,
/// contact order preserved. `_tr_id`/`_dialect` select nothing today — they
/// keep the decide signature stable for the flows where a second dialect
/// forks policy; the wire-shape fork itself lives in `render`.
pub(crate) fn decide_chg(gathered: Vec<GatheredContact>, _tr_id: u128, _dialect: Dialect) -> Vec<ClientOut> {
    gathered.into_iter().map(|contact| {
        let endpoint_id = EndpointId::from_email_addr(contact.network_id_email.email.clone());
        let endpoint_guid = endpoint_id.endpoint_guid.expect("to be here");

        ClientOut::ContactPresence {
            user: contact.network_id_email,
            status: PresenceStatus::NLN,
            display_name: contact.display_name.map(|name| DisplayName::new(name) ).unwrap_or_default(),
            avatar: contact.avatar,
            endpoint_data: EndpointData::new(Some(endpoint_guid), ClientCapabilities::default()),
        }
    }).collect()
}

/// Render step: `ClientOut` -> wire commands, byte-identical to the pre-D1
/// handler for MSNP18.
pub(crate) fn render(out: ClientOut, tr_id: u128, dialect: Dialect) -> Vec<NotificationServerCommand> {
    match out {
        ClientOut::Command(command) => vec![command],
        ClientOut::ContactPresence { user, status, display_name, avatar, endpoint_data } => {
            match dialect {
                // Seam site (presence render): the MSNP21 presence payload /
                // UBX endpoint-data shape differs, and MSNP21 serves SDG/UUM
                // over the NS with no switchboard (§7 M1) — a v21 arm lands
                // there, never a `_` wildcard here.
                Dialect::Msnp18 => {
                    vec![
                        NotificationServerCommand::ILN(IlnServer{
                            tr_id,
                            presence_status: status,
                            target_user: user.clone(),
                            via: None,
                            display_name,
                            client_capabilities: Default::default(),
                            avatar,
                            badge_url: None,
                        }),
                        //If we don't set the EndpointData here, we don't get P2P Transport Requests because the client has no information about the presence of this Endpoint.
                        NotificationServerCommand::UBX(UbxServer {
                            target_user: user,
                            via: None,
                            payload: UbxPayload::ExtendedPresence(ExtendedPresenceContent {
                                psm: "".to_string(),
                                current_media: "".to_string(),
                                endpoint_data,
                                private_endpoint_data: None,
                            }),
                        }),
                    ]
                },
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use msnp::msnp::models::contact_list::ContactList;
    use msnp::msnp::notification::command::chg::ChgClient;
    use msnp::msnp::notification::command::command::NotificationServerCommand;
    use msnp::msnp::notification::models::dialect::Dialect;
    use msnp::msnp::notification::models::endpoint_data::EndpointData;
    use msnp::msnp::raw_command_parser::RawCommand;
    use msnp::shared::models::capabilities::ClientCapabilities;
    use msnp::shared::models::display_name::DisplayName;
    use msnp::shared::models::email_address::EmailAddress;
    use msnp::shared::models::endpoint_id::EndpointId;
    use msnp::shared::models::msn_user::MsnUser;
    use msnp::shared::models::network_id::NetworkId;
    use msnp::shared::models::network_id_email::NetworkIdEmail;
    use msnp::shared::models::presence_status::PresenceStatus;
    use msnp::shared::models::ticket_token::TicketToken;
    use msnp::shared::traits::{IntoBytes, TryFromRawCommand};

    use super::{decide_chg, handle_chg, render, GatheredContact};
    use crate::notification::models::client_out::ClientOut;
    use crate::notification::models::session_state::SessionState;
    use crate::tachyon::client::tachyon_client::TachyonClient;
    use crate::tachyon::config::tachyon_config::TachyonConfig;

    fn gathered(email: &str, display_name: Option<&str>) -> GatheredContact {
        GatheredContact {
            network_id_email: NetworkIdEmail {
                network_id: NetworkId::WindowsLive,
                email: EmailAddress::from_str(email).unwrap(),
            },
            display_name: display_name.map(|name| name.to_string()),
            avatar: None,
        }
    }

    #[test]
    fn decide_maps_gathered_contacts_to_presence_outs_in_order() {
        let gathered = vec![
            gathered("aeon@test.com", Some("Aeon Test")),
            gathered("peer@test.com", None),
        ];

        let outs = decide_chg(gathered, 6, Dialect::Msnp18);

        assert_eq!(outs.len(), 2);

        let ClientOut::ContactPresence { user, status, display_name, avatar, endpoint_data } = &outs[0] else {
            panic!("expected a contact presence for the first gathered contact");
        };
        assert_eq!(user.network_id, NetworkId::WindowsLive);
        assert_eq!(user.email, EmailAddress::from_str("aeon@test.com").unwrap());
        assert_eq!(*status, PresenceStatus::NLN);
        assert_eq!(*display_name, DisplayName::new("Aeon Test".to_string()));
        assert!(avatar.is_none());
        let expected_guid = EndpointId::from_email_addr(EmailAddress::from_str("aeon@test.com").unwrap()).endpoint_guid.unwrap();
        assert_eq!(endpoint_data.machine_guid.as_ref().unwrap().to_string(), expected_guid.to_string());
        assert_eq!(endpoint_data.capabilities.to_string(), ClientCapabilities::default().to_string());

        let ClientOut::ContactPresence { user, display_name, .. } = &outs[1] else {
            panic!("expected a contact presence for the second gathered contact");
        };
        assert_eq!(user.email, EmailAddress::from_str("peer@test.com").unwrap());
        assert_eq!(*display_name, DisplayName::default());
    }

    #[test]
    fn render_contact_presence_pins_the_iln_and_ubx_wire_bytes() {
        let email = EmailAddress::from_str("aeon@test.com").unwrap();
        let endpoint_guid = EndpointId::from_email_addr(email.clone()).endpoint_guid.unwrap();

        let out = ClientOut::ContactPresence {
            user: NetworkIdEmail { network_id: NetworkId::WindowsLive, email },
            status: PresenceStatus::NLN,
            display_name: DisplayName::new("Aeon Test".to_string()),
            avatar: None,
            endpoint_data: EndpointData::new(Some(endpoint_guid), ClientCapabilities::default()),
        };

        let rendered = render(out, 6, Dialect::Msnp18);
        assert_eq!(rendered.len(), 2);

        let [iln, ubx] = <[NotificationServerCommand; 2]>::try_from(rendered).ok().expect("the presence pair");

        let NotificationServerCommand::ILN(iln) = iln else {
            panic!("expected the ILN first");
        };
        assert_eq!(String::from_utf8(iln.into_bytes()).unwrap(), "ILN 6 NLN 1:aeon@test.com Aeon%20Test 2754973748:3734755248 0\r\n");

        let NotificationServerCommand::UBX(ubx) = ubx else {
            panic!("expected the UBX second");
        };
        assert_eq!(String::from_utf8(ubx.into_bytes()).unwrap(), "UBX 1:aeon@test.com 176\r\n<Data><PSM></PSM><CurrentMedia></CurrentMedia><EndpointData id=\"{1F8BFCBF-4F72-587D-8A18-489C017B448B}\"><Capabilities>2754973748:3734755248</Capabilities></EndpointData></Data>");
    }

    #[test]
    fn render_passes_escape_hatch_commands_through_untouched() {
        let rendered = render(ClientOut::Command(NotificationServerCommand::OUT), 6, Dialect::Msnp18);

        assert_eq!(rendered.len(), 1);
        assert!(matches!(rendered[0], NotificationServerCommand::OUT));
    }

    #[tokio::test]
    async fn a_second_chg_with_the_flag_already_flipped_only_acks() {
        let (snd, mut rcv) = tokio::sync::mpsc::channel::<NotificationServerCommand>(10);
        let (kill_snd, kill_recv) = tokio::sync::broadcast::channel::<()>(1);

        let msn_user = MsnUser::new(EndpointId::new(EmailAddress::from_str("aeon@test.com").unwrap(), None));
        let tachyon_client = TachyonClient::new(TachyonConfig::default(), msn_user, TicketToken::default(), snd.clone(), kill_snd, kill_recv, ContactList::default());
        // built, never connected to: with the flag flipped the shell must not touch it
        let matrix_client = matrix_sdk::Client::builder().homeserver_url("http://127.0.0.1:1").build().await.unwrap();

        let mut session_state = SessionState::new();
        session_state.needs_initial_presence = false;

        let chg = ChgClient::try_from_raw(RawCommand::from_str("CHG 6 NLN 0:0 0").unwrap()).unwrap();
        handle_chg(chg, &mut session_state, tachyon_client, matrix_client, snd).await.unwrap();

        let ack = rcv.recv().await.unwrap();
        assert!(matches!(ack, NotificationServerCommand::CHG(_)));

        // the sender moved into handle_chg is dropped by now: the channel closing
        // with just the ack proves no presence fan-out happened.
        assert!(rcv.recv().await.is_none());
    }
}
