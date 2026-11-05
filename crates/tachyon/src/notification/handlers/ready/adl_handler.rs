use crate::matrix::extensions::msn_user_resolver::{FindRoomFromEmail, ToMsnUser};
use crate::notification::models::client_out::ClientOut;
use crate::tachyon::client::tachyon_client::TachyonClient;
use crate::tachyon::identifiers::is_sha1::IsSha1;
use log::{debug, warn};
use matrix_sdk::Client;
use msnp::msnp::notification::command::adl::AdlClient;
use msnp::msnp::notification::command::command::NotificationServerCommand;
use msnp::msnp::notification::command::nln::NlnServer;
use msnp::msnp::notification::command::ubx::{ExtendedPresenceContent, UbxPayload, UbxServer};
use msnp::shared::models::display_name::DisplayName;
use msnp::shared::models::msn_object::MsnObject;
use msnp::shared::models::network_id_email::NetworkIdEmail;
use msnp::shared::models::presence_status::PresenceStatus;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::time::sleep;
use msnp::msnp::notification::models::endpoint_data::EndpointData;
use msnp::shared::models::capabilities::ClientCapabilities;
use msnp::shared::models::endpoint_id::EndpointId;

/// Gather output for one ADL-listed contact: plain data only — every async
/// lookup (room, display name, avatar) is already resolved (§3.1, §10: the
/// gather stays per-flow, over only the fields this flow reads).
/// Deliberately NOT shared with chg's gather struct: the flows are separate
/// and adl's wire answers differ (§3.2).
pub(crate) struct GatheredContact {
    pub(crate) network_id_email: NetworkIdEmail,
    pub(crate) display_name: Option<String>,
    pub(crate) avatar: Option<MsnObject>,
}

/// Pattern B shell (§3.1, chg precedent): the ack ordering interleaves with
/// matrix IO and the fan-out runs in a detached spawn, so the Sender stays
/// in the shell. Spawn placement is shell policy, preserved exactly (§3.6),
/// including the 1s delay and the per-contact gather→send interleave: a
/// send error (client gone) aborts before the NEXT contact's lookups run,
/// exactly as pre-D2. The contact-list mutation and the OK ack keep their
/// pre-D2 order and `?` policy.
pub async fn handle_adl(command: AdlClient, tachyon_client: TachyonClient, matrix_client: Client, command_sender: Sender<NotificationServerCommand>) -> Result<(), anyhow::Error>  {
    debug!("ADL: {:?}", &command);

    let contacts = command.payload.get_contacts()?;

    {
    let mut contact_list = tachyon_client.get_contact_list().lock().unwrap();
        contact_list.add_contacts(contacts.clone(), command.payload.is_initial());
    }

    command_sender.send(NotificationServerCommand::OK(command.get_ok_response("ADL"))).await?;

    tokio::spawn( async move {

        sleep(Duration::from_millis(1000)).await;
        //Hardcoded presence to Online
        //TODO: implement real presence
        for contact in contacts {

            if !contact.email_address.is_sha1_imprecise() {
                continue;
            }

            // Gather: async IO only, per contact — the interleave with the
            // sends below is load-bearing and preserved from pre-D2.
            let found_room = matrix_client.find_room_from_email(&contact.email_address);

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
            // (§1.2, chg's identical unwrap retired in D1). Log it, announce
            // the contact without avatar, carry on with the remaining ones.
            let avatar = if let Ok(Some(room)) = &found_room {
                match tachyon_client.get_avatar_as_msn_object(room.room_id()).await {
                    Ok(avatar) => avatar,
                    Err(error) => {
                        warn!("MSNP|NS: unable to build the avatar msnobj for {}: {}", contact.email_address.as_str(), error);
                        None
                    },
                }
            } else { None };


            let gathered = GatheredContact {
                network_id_email: NetworkIdEmail {
                    network_id: contact.network_id.clone(),
                    email: contact.email_address.clone(),
                },
                display_name,
                avatar,
            };

            // Decide (pure), then send through the existing sink, NLN before
            // UBX, in Vec order.
            for out in decide_adl(gathered) {
                let server_command = match out {
                    // adl rides Command(..) — its presence pair is NLN+UBX,
                    // NOT chg's ILN+UBX ContactPresence render (§3.2).
                    ClientOut::Command(command) => command,
                    ClientOut::ContactPresence { .. } => {
                        warn!("MSNP|NS: the adl fan-out produced ContactPresence; only chg renders presence — dropped");
                        continue;
                    }
                    // Canary precedent (the ContactPresence arm above): loud
                    // warn + skip, unreachable by construction — the adl fan-out
                    // never produces the auth trigger.
                    ClientOut::DirectToWebLogin { .. } => {
                        warn!("MSNP|NS: the adl fan-out produced DirectToWebLogin; only the auth seam renders it — dropped");
                        continue;
                    }
                };

                // Err = receiver dropped, client gone: stop the fan-out (§8 D2).
                if let Err(error) = command_sender.send(server_command).await {
                    warn!("MSNP|NS: presence fan-out aborted, client gone: {}", error);
                    return;
                }
            }
        }
    });
    Ok(())
}

/// Pure decision (§3.1): one NLN + UBX pair per gathered contact, NLN first
/// — the pre-D2 handler's exact wire order. Escape hatch, deliberately
/// (§3.2): adl announces contacts with NLN (no tr_id — the server pushes it
/// unsolicited), while chg's ContactPresence renders ILN (tr_id-bound, the
/// answer to the client's own CHG). That operand-level difference is the
/// demonstrated reason ContactPresence does NOT fit adl, so both commands
/// ride Command(..) and never touch chg's render.
pub(crate) fn decide_adl(gathered: GatheredContact) -> Vec<ClientOut> {
    let endpoint_id = EndpointId::from_email_addr(gathered.network_id_email.email.clone());
    let endpoint_guid = endpoint_id.endpoint_guid.expect("to be here");

    vec![
        ClientOut::Command(NotificationServerCommand::NLN(NlnServer{
            presence_status: PresenceStatus::NLN,
            target_user: gathered.network_id_email.clone(),
            via: None,
            display_name: gathered.display_name.map(|name| DisplayName::new(name) ).unwrap_or_default(),
            client_capabilities: Default::default(),
            avatar: gathered.avatar,
            badge_url: None,
        })),
        //If we don't set the EndpointData here, we don't get P2P Transport Requests because the client has no information about the presence of this Endpoint.
        // Err = receiver dropped, client gone: stop the fan-out (§8 D2).
        ClientOut::Command(NotificationServerCommand::UBX(UbxServer {
            target_user: gathered.network_id_email,
            via: None,
            payload: UbxPayload::ExtendedPresence(ExtendedPresenceContent {
                psm: "".to_string(),
                current_media: "".to_string(),
                endpoint_data: EndpointData::new(Some(endpoint_guid), ClientCapabilities::default()),
                private_endpoint_data: None,
            }),
        })),
    ]
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use msnp::msnp::models::contact_list::ContactList;
    use msnp::msnp::notification::command::adl::AdlClient;
    use msnp::msnp::notification::command::command::NotificationServerCommand;
    use msnp::msnp::notification::models::adl_payload::ADLPayload;
    use msnp::shared::models::display_name::DisplayName;
    use msnp::shared::models::email_address::EmailAddress;
    use msnp::shared::models::endpoint_id::EndpointId;
    use msnp::shared::models::msn_user::MsnUser;
    use msnp::shared::models::network_id::NetworkId;
    use msnp::shared::models::network_id_email::NetworkIdEmail;
    use msnp::shared::models::presence_status::PresenceStatus;
    use msnp::shared::models::ticket_token::TicketToken;
    use msnp::shared::traits::{IntoBytes, TryFromBytes};

    use super::{decide_adl, handle_adl, GatheredContact};
    use crate::notification::models::client_out::ClientOut;
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
    fn decide_announces_a_gathered_contact_with_the_nln_then_ubx_pair() {
        let outs = decide_adl(gathered("aeon@test.com", Some("Aeon Test")));
        assert_eq!(outs.len(), 2);

        let ClientOut::Command(command) = &outs[0] else {
            panic!("adl decides bare wire commands");
        };
        let NotificationServerCommand::NLN(nln) = command else {
            panic!("expected the NLN first");
        };
        assert_eq!(nln.presence_status, PresenceStatus::NLN);
        assert_eq!(nln.target_user.email, EmailAddress::from_str("aeon@test.com").unwrap());
        assert_eq!(nln.display_name, DisplayName::new("Aeon Test".to_string()));
        assert!(nln.via.is_none());
        assert!(nln.avatar.is_none());

        let ClientOut::Command(command) = &outs[1] else {
            panic!("adl decides bare wire commands");
        };
        let NotificationServerCommand::UBX(ubx) = command else {
            panic!("expected the UBX second");
        };
        assert_eq!(ubx.target_user.email, EmailAddress::from_str("aeon@test.com").unwrap());
    }

    #[test]
    fn decide_pins_the_nln_and_ubx_wire_bytes() {
        let mut outs = decide_adl(gathered("aeon@test.com", Some("Aeon Test"))).into_iter();

        let ClientOut::Command(nln) = outs.next().expect("the NLN first") else {
            panic!("adl decides bare wire commands");
        };
        assert_eq!(String::from_utf8(nln.into_bytes()).unwrap(), "NLN NLN 1:aeon@test.com Aeon%20Test 2754973748:3734755248 0\r\n");

        let ClientOut::Command(ubx) = outs.next().expect("the UBX second") else {
            panic!("adl decides bare wire commands");
        };
        assert_eq!(String::from_utf8(ubx.into_bytes()).unwrap(), "UBX 1:aeon@test.com 176\r\n<Data><PSM></PSM><CurrentMedia></CurrentMedia><EndpointData id=\"{1F8BFCBF-4F72-587D-8A18-489C017B448B}\"><Capabilities>2754973748:3734755248</Capabilities></EndpointData></Data>");

        assert!(outs.next().is_none());
    }

    #[test]
    fn a_contact_without_a_resolved_display_name_gets_the_default_one() {
        let table = vec![
            ("peer@test.com", None),
            ("aeon@test.com", None),
        ];

        for (email, display_name) in table {
            let outs = decide_adl(gathered(email, display_name));

            let ClientOut::Command(command) = &outs[0] else {
                panic!("adl decides bare wire commands");
            };
            let NotificationServerCommand::NLN(nln) = command else {
                panic!("expected the NLN first");
            };
            assert_eq!(nln.display_name, DisplayName::default());
            assert_eq!(nln.target_user.email, EmailAddress::from_str(email).unwrap());
        }
    }

    #[test]
    fn an_avatar_lookup_failure_still_announces_the_contact_with_the_full_pair() {
        // §1.2: the gather now warns and carries None instead of unwrapping a
        // failed avatar lookup; decide must still emit the whole NLN+UBX pair,
        // so one bad lookup never strands this contact or the ones after it.
        let outs = decide_adl(gathered("aeon@test.com", Some("Aeon Test")));
        assert_eq!(outs.len(), 2);

        let ClientOut::Command(nln) = &outs[0] else {
            panic!("adl decides bare wire commands");
        };
        let NotificationServerCommand::NLN(nln) = nln else {
            panic!("expected the NLN first");
        };
        assert!(nln.avatar.is_none());

        let ClientOut::Command(ubx) = &outs[1] else {
            panic!("adl decides bare wire commands");
        };
        assert!(matches!(ubx, NotificationServerCommand::UBX(_)));
    }

    #[tokio::test]
    async fn adl_stores_the_contacts_and_acks_before_the_fan_out_is_spawned() {
        let (snd, mut rcv) = tokio::sync::mpsc::channel::<NotificationServerCommand>(10);
        let (kill_snd, kill_recv) = tokio::sync::broadcast::channel::<()>(1);

        let msn_user = MsnUser::new(EndpointId::new(EmailAddress::from_str("aeon@test.com").unwrap(), None));
        let tachyon_client = TachyonClient::new(TachyonConfig::default(), msn_user, TicketToken::default(), snd.clone(), kill_snd, kill_recv, ContactList::default());
        // built, never connected to: a non-sha1 contact is skipped by the
        // fan-out before any matrix lookup, so this test stays socket-free
        let matrix_client = matrix_sdk::Client::builder().homeserver_url("http://127.0.0.1:1").build().await.unwrap();

        let xml = "<ml><d n=\"test.com\"><c n=\"peer\" l=\"4\" /></d></ml>";
        let command = AdlClient { tr_id: 4, payload: ADLPayload::try_from_bytes(xml.as_bytes().to_vec()).unwrap() };

        handle_adl(command, tachyon_client.clone(), matrix_client, snd).await.unwrap();

        let ack = rcv.recv().await.unwrap();
        let NotificationServerCommand::OK(ok) = ack else {
            panic!("expected the ADL ack");
        };
        assert_eq!(ok.operand, "ADL");
        assert_eq!(ok.tr_id, 4);

        assert_eq!(tachyon_client.get_contact_list().lock().unwrap().contact_list.len(), 1);
    }
}
