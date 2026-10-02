//! Unit tests for the event drivers, on the in-process plugin.

use tfar_ts3_host::{ChannelId, ClientId, ConnectStatus, ServerConnectionHandlerId};

use super::*;
use crate::mock_host::in_process_plugin::{PluginEvent, start_in_process_host, take_events};

fn status(server: u64, status: ConnectStatus, own_client: Option<u16>) -> PluginEvent {
    PluginEvent::ConnectStatus {
        server,
        status: status.to_raw(),
        own_client,
    }
}

#[test]
fn connecting_a_server_raises_teamspeaks_sequence() {
    let host = start_in_process_host();
    take_events();
    let setup = ServerSetup {
        channel_names: vec!["Lobby".to_owned(), "Briefing".to_owned()],
        ..ServerSetup::default()
    };
    let server = host.connect_server(&setup).unwrap();
    assert_eq!(server, ServerConnectionHandlerId(1));
    assert_eq!(host.current_server(), server);
    assert_eq!(
        take_events(),
        vec![
            PluginEvent::CurrentServerChanged(1),
            status(1, ConnectStatus::Connecting, None),
            status(1, ConnectStatus::Connected, None),
            status(1, ConnectStatus::ConnectionEstablishing, None),
            PluginEvent::NewChannel {
                server: 1,
                channel: 1,
                parent: 0
            },
            PluginEvent::NewChannel {
                server: 1,
                channel: 2,
                parent: 0
            },
            status(1, ConnectStatus::ConnectionEstablished, Some(1)),
        ]
    );
    host.with_fake_teamspeak(|fake| {
        let fake_server = fake.server(server).unwrap();
        assert_eq!(
            fake_server.connect_status,
            ConnectStatus::ConnectionEstablished
        );
        assert_eq!(fake_server.channels.len(), 2);
        let own = fake_server.clients.get(&fake_server.own_client).unwrap();
        assert_eq!(own.nickname, "Mock Player");
        assert_eq!(own.channel, ChannelId(1));
    });
    assert!(host.leak_report().is_clean());
}

#[test]
fn an_empty_channel_list_connects_with_a_default_channel() {
    let host = start_in_process_host();
    let setup = ServerSetup {
        channel_names: Vec::new(),
        ..ServerSetup::default()
    };
    let server = host.connect_server(&setup).unwrap();
    host.with_fake_teamspeak(|fake| {
        let names: Vec<String> = fake
            .server(server)
            .unwrap()
            .channels
            .values()
            .map(|c| c.name.clone())
            .collect();
        assert_eq!(names, vec!["Default Channel".to_owned()]);
    });
}

#[test]
fn clients_join_move_and_leave_with_the_state_set_before_each_callback() {
    let host = start_in_process_host();
    let setup = ServerSetup {
        channel_names: vec!["Lobby".to_owned(), "Briefing".to_owned()],
        ..ServerSetup::default()
    };
    host.connect_server(&setup).unwrap();
    take_events();
    let joined = host.client_joins("Rifleman", ChannelId(1)).unwrap();
    assert_eq!(joined, ClientId(2));
    host.client_moves(joined, ChannelId(2)).unwrap();
    host.client_leaves(joined).unwrap();
    let movement = |old_channel, new_channel, visibility, message: &str, nickname: Option<&str>| {
        PluginEvent::ClientMove {
            server: 1,
            client: 2,
            old_channel,
            new_channel,
            visibility,
            message: message.to_owned(),
            nickname_seen: nickname.map(str::to_owned),
        }
    };
    assert_eq!(
        take_events(),
        vec![
            movement(0, 1, 0, "", Some("Rifleman")),
            movement(1, 2, 1, "", Some("Rifleman")),
            movement(2, 0, 2, "leaving", None),
        ]
    );
    host.with_fake_teamspeak(|fake| {
        assert!(fake.client(ServerConnectionHandlerId(1), joined).is_none());
    });
    assert!(host.leak_report().is_clean());
}

#[test]
fn talking_changes_the_flag_without_a_callback() {
    let host = start_in_process_host();
    host.connect_server(&ServerSetup::default()).unwrap();
    let client = host.client_joins("Radio Operator", ChannelId(1)).unwrap();
    take_events();
    host.set_talking(client, true).unwrap();
    assert_eq!(take_events(), Vec::new());
    let server = host.current_server();
    let is_talking =
        host.with_fake_teamspeak(|fake| fake.client(server, client).unwrap().is_talking);
    assert!(is_talking);
}

#[test]
fn metadata_is_set_and_announced_by_the_client_itself() {
    let host = start_in_process_host();
    host.connect_server(&ServerSetup::default()).unwrap();
    let client = host.client_joins("Medic", ChannelId(1)).unwrap();
    take_events();
    host.set_client_metadata(client, "<TFAR>1.0</TFAR>")
        .unwrap();
    assert_eq!(
        take_events(),
        vec![PluginEvent::UpdateClient {
            server: 1,
            client: 2,
            invoker: 2,
            invoker_name: "Medic".to_owned(),
        }]
    );
}

#[test]
fn a_plugin_command_arrives_with_its_invoker() {
    let host = start_in_process_host();
    host.connect_server(&ServerSetup::default()).unwrap();
    let client = host.client_joins("Pilot", ChannelId(1)).unwrap();
    take_events();
    host.deliver_plugin_command(client, "task_force_radio", "VERSION\t1.0")
        .unwrap();
    assert_eq!(
        take_events(),
        vec![PluginEvent::PluginCommand {
            server: 1,
            plugin_name: "task_force_radio".to_owned(),
            command: "VERSION\t1.0".to_owned(),
            invoker: 2,
            invoker_name: "Pilot".to_owned(),
            invoker_unique_identifier: "mock-client-2=".to_owned(),
        }]
    );
}

#[test]
fn drivers_without_a_server_or_with_unknown_ids_are_errors() {
    let host = start_in_process_host();
    assert!(matches!(
        host.client_joins("Nobody", ChannelId(1)),
        Err(Error::NoCurrentServer)
    ));
    host.connect_server(&ServerSetup::default()).unwrap();
    assert!(matches!(
        host.client_joins("Nobody", ChannelId(9)),
        Err(Error::UnknownChannel {
            channel: ChannelId(9)
        })
    ));
    assert!(matches!(
        host.client_moves(ClientId(9), ChannelId(1)),
        Err(Error::UnknownClient {
            client: ClientId(9)
        })
    ));
    assert!(matches!(
        host.deliver_plugin_command(ClientId(9), "x", "y"),
        Err(Error::UnknownClient { .. })
    ));
    assert!(matches!(
        host.set_current_server(ServerConnectionHandlerId(5)),
        Err(Error::UnknownServer { .. })
    ));
    assert!(matches!(
        host.deliver_plugin_command(ClientId(1), "x", "a\0b"),
        Err(Error::InteriorNul {
            argument: "command"
        })
    ));
}

#[test]
fn a_second_server_becomes_current_and_tabs_can_switch_and_disconnect() {
    let host = start_in_process_host();
    let first = host.connect_server(&ServerSetup::default()).unwrap();
    let second = host.connect_server(&ServerSetup::default()).unwrap();
    assert_eq!(second, ServerConnectionHandlerId(2));
    assert_eq!(host.current_server(), second);
    take_events();
    host.set_current_server(first).unwrap();
    host.disconnect_server(second).unwrap();
    assert_eq!(
        take_events(),
        vec![
            PluginEvent::CurrentServerChanged(1),
            status(2, ConnectStatus::Disconnected, None),
        ]
    );
    host.with_fake_teamspeak(|fake| {
        let disconnected = fake.server(second).unwrap();
        assert_eq!(disconnected.connect_status, ConnectStatus::Disconnected);
        assert!(disconnected.clients.is_empty());
    });
}
