//! Layout tests: the `ts3plugin-sys` `Ts3Functions` table and the SDK values this crate passes,
//! checked against the official API 26 headers in `sdk_headers/`, parsed as text at test time.

use std::collections::HashMap;
use std::mem::{offset_of, size_of};
use std::path::Path;

use ts3plugin_sys::public_definitions::Ts3Vector as SdkVector;
use ts3plugin_sys::ts3functions::Ts3Functions;

use crate::{
    ChannelProperty, ClientProperty, ConnectStatus, InfoItem, LogLevel, PluginCommandTarget,
    ServerProperty, TalkStatus, Visibility,
};

/// Every `TS3Functions` slot TFAR calls: the SDK name and the `ts3plugin-sys` field offset.
///
/// The union of both behaviour specs, §4.
const USED_FUNCTIONS: &[(&str, usize)] = &[
    (
        "getErrorMessage",
        offset_of!(Ts3Functions, get_error_message),
    ),
    ("freeMemory", offset_of!(Ts3Functions, free_memory)),
    ("logMessage", offset_of!(Ts3Functions, log_message)),
    (
        "getCurrentServerConnectionHandlerID",
        offset_of!(Ts3Functions, get_current_server_connection_handler_id),
    ),
    (
        "getServerConnectionHandlerList",
        offset_of!(Ts3Functions, get_server_connection_handler_list),
    ),
    (
        "getConnectionStatus",
        offset_of!(Ts3Functions, get_connection_status),
    ),
    ("getClientID", offset_of!(Ts3Functions, get_client_id)),
    ("getClientList", offset_of!(Ts3Functions, get_client_list)),
    (
        "getChannelClientList",
        offset_of!(Ts3Functions, get_channel_client_list),
    ),
    ("getChannelList", offset_of!(Ts3Functions, get_channel_list)),
    (
        "getChannelOfClient",
        offset_of!(Ts3Functions, get_channel_of_client),
    ),
    (
        "getClientVariableAsString",
        offset_of!(Ts3Functions, get_client_variable_as_string),
    ),
    (
        "getClientVariableAsInt",
        offset_of!(Ts3Functions, get_client_variable_as_int),
    ),
    (
        "getClientSelfVariableAsInt",
        offset_of!(Ts3Functions, get_client_self_variable_as_int),
    ),
    (
        "setClientSelfVariableAsString",
        offset_of!(Ts3Functions, set_client_self_variable_as_string),
    ),
    (
        "setClientSelfVariableAsInt",
        offset_of!(Ts3Functions, set_client_self_variable_as_int),
    ),
    (
        "flushClientSelfUpdates",
        offset_of!(Ts3Functions, flush_client_self_updates),
    ),
    (
        "getChannelVariableAsString",
        offset_of!(Ts3Functions, get_channel_variable_as_string),
    ),
    (
        "getServerVariableAsString",
        offset_of!(Ts3Functions, get_server_variable_as_string),
    ),
    (
        "requestClientMove",
        offset_of!(Ts3Functions, request_client_move),
    ),
    (
        "requestMuteClients",
        offset_of!(Ts3Functions, request_mute_clients),
    ),
    (
        "requestUnmuteClients",
        offset_of!(Ts3Functions, requset_unmute_clients),
    ),
    (
        "sendPluginCommand",
        offset_of!(Ts3Functions, send_plugin_command),
    ),
    ("playWaveFile", offset_of!(Ts3Functions, play_wave_file)),
    (
        "systemset3DSettings",
        offset_of!(Ts3Functions, systemset3d_settings),
    ),
    (
        "systemset3DListenerAttributes",
        offset_of!(Ts3Functions, systemset3d_listener_attributes),
    ),
    (
        "channelset3DAttributes",
        offset_of!(Ts3Functions, channelset3d_attributes),
    ),
    (
        "getPreProcessorConfigValue",
        offset_of!(Ts3Functions, get_pre_processor_config_value),
    ),
    (
        "setPreProcessorConfigValue",
        offset_of!(Ts3Functions, set_pre_processor_config_value),
    ),
    ("getPluginPath", offset_of!(Ts3Functions, get_plugin_path)),
    ("getConfigPath", offset_of!(Ts3Functions, get_config_path)),
    (
        "printMessageToCurrentTab",
        offset_of!(Ts3Functions, print_message_to_current_tab),
    ),
];

/// The text of `sdk_headers/<name>`.
fn header(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("sdk_headers")
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

/// `text` without `/* … */` and `// …` comments.
fn without_comments(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while !rest.is_empty() {
        if let Some(after) = rest.strip_prefix("/*") {
            rest = after.find("*/").map_or("", |end| &after[end + 2..]);
        } else if let Some(after) = rest.strip_prefix("//") {
            rest = after.find('\n').map_or("", |end| &after[end..]);
        } else {
            let next = rest.chars().next().unwrap();
            out.push(next);
            rest = &rest[next.len_utf8()..];
        }
    }
    out
}

/// The function pointer names of `struct TS3Functions`, in declaration (ABI) order.
fn ts3_function_names() -> Vec<String> {
    let text = without_comments(&header("ts3_functions.h"));
    let body = text
        .split("struct TS3Functions {")
        .nth(1)
        .unwrap()
        .split("};")
        .next()
        .unwrap();
    let mut names = Vec::new();
    let mut rest = body;
    while let Some(start) = rest.find("(*") {
        let after = &rest[start + 2..];
        let end = after.find(')').unwrap();
        names.push(after[..end].trim().to_owned());
        rest = &after[end..];
    }
    names
}

/// Every enumerator of the headers that define enums, with its value.
fn enum_values() -> HashMap<String, i64> {
    let mut values = HashMap::new();
    for name in [
        "plugin_definitions.h",
        "public_definitions.h",
        "public_rare_definitions.h",
        "logtypes.h",
    ] {
        let text = without_comments(&header(name));
        for block in text.split("enum").skip(1) {
            let Some(open) = block.find('{') else {
                continue;
            };
            if block[..open].contains(';') || block[..open].contains(')') {
                continue; // `enum X` used as a type, not a definition
            }
            let close = block.find('}').unwrap();
            let mut next = 0_i64;
            for entry in block[open + 1..close].split(',') {
                let entry: String = entry
                    .lines()
                    .filter(|line| !line.trim_start().starts_with('#'))
                    .collect();
                let entry = entry.trim();
                if entry.is_empty() {
                    continue;
                }
                let (enumerator, value) = match entry.split_once('=') {
                    Some((enumerator, value)) => {
                        (enumerator.trim(), parse_value(value.trim(), &values))
                    }
                    None => (entry, next),
                };
                values.insert(enumerator.to_owned(), value);
                next = value + 1;
            }
        }
    }
    values
}

/// An enumerator value as the headers write them: terms joined by `|`, each a `<<` shift or a
/// single decimal or hex literal or the name of an enumerator defined before.
fn parse_value(text: &str, known: &HashMap<String, i64>) -> i64 {
    text.split('|')
        .map(|term| match term.split_once("<<") {
            Some((base, shift)) => parse_atom(base, known) << parse_atom(shift, known),
            None => parse_atom(term, known),
        })
        .fold(0, |all, term| all | term)
}

/// A decimal or hex literal, or the name of an enumerator defined before, in optional brackets.
fn parse_atom(text: &str, known: &HashMap<String, i64>) -> i64 {
    let text = text
        .trim()
        .trim_start_matches('(')
        .trim_end_matches(')')
        .trim();
    if let Some(hex) = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
        return i64::from_str_radix(hex.trim_end_matches(['u', 'U', 'l', 'L']), 16).unwrap();
    }
    text.parse().unwrap_or_else(|_| {
        *known
            .get(text)
            .unwrap_or_else(|| panic!("unknown value {text}"))
    })
}

fn value(values: &HashMap<String, i64>, enumerator: &str) -> i64 {
    *values
        .get(enumerator)
        .unwrap_or_else(|| panic!("{enumerator} is not in the SDK headers"))
}

#[test]
fn the_function_table_is_one_pointer_per_header_slot() {
    let names = ts3_function_names();
    assert_eq!(names.len(), 230, "API 26 declares 230 function pointers");
    assert_eq!(
        size_of::<Ts3Functions>(),
        names.len() * size_of::<*const ()>()
    );
}

#[test]
fn every_function_tfar_calls_sits_at_its_header_offset() {
    let names = ts3_function_names();
    let mismatches: Vec<String> = USED_FUNCTIONS
        .iter()
        .filter_map(|&(name, offset)| {
            let index = names.iter().position(|candidate| candidate == name)?;
            let expected = index * size_of::<*const ()>();
            (offset != expected)
                .then(|| format!("{name}: ts3plugin-sys {offset}, header {expected}"))
        })
        .collect();
    assert!(mismatches.is_empty(), "misplaced slots: {mismatches:?}");
    let missing: Vec<&str> = USED_FUNCTIONS
        .iter()
        .map(|&(name, _)| name)
        .filter(|name| !names.iter().any(|n| n == name))
        .collect();
    assert!(missing.is_empty(), "not in the header: {missing:?}");
}

#[test]
fn the_3d_vector_is_three_floats_as_in_the_header() {
    let text = without_comments(&header("public_definitions.h"));
    let body = text
        .split("TS3_VECTOR;")
        .next()
        .unwrap()
        .rsplit('{')
        .next()
        .unwrap();
    let fields: Vec<String> = body
        .split(';')
        .map(|field| field.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect();
    assert_eq!(
        fields[..3],
        [
            "float x".to_owned(),
            "float y".to_owned(),
            "float z".to_owned()
        ]
    );
    assert_eq!(size_of::<SdkVector>(), 12);
    assert_eq!(
        (
            offset_of!(SdkVector, x),
            offset_of!(SdkVector, y),
            offset_of!(SdkVector, z)
        ),
        (0, 4, 8)
    );
}

#[test]
fn every_property_flag_equals_its_header_value() {
    let values = enum_values();
    let client = [
        (ClientProperty::UniqueIdentifier, "CLIENT_UNIQUE_IDENTIFIER"),
        (ClientProperty::Nickname, "CLIENT_NICKNAME"),
        (ClientProperty::FlagTalking, "CLIENT_FLAG_TALKING"),
        (ClientProperty::InputMuted, "CLIENT_INPUT_MUTED"),
        (ClientProperty::OutputMuted, "CLIENT_OUTPUT_MUTED"),
        (ClientProperty::InputDeactivated, "CLIENT_INPUT_DEACTIVATED"),
        (ClientProperty::MetaData, "CLIENT_META_DATA"),
        (ClientProperty::IsMuted, "CLIENT_IS_MUTED"),
    ];
    for (property, enumerator) in client {
        assert_eq!(
            i64::try_from(property.flag()).unwrap(),
            value(&values, enumerator),
            "{enumerator}"
        );
    }
    assert_eq!(
        i64::try_from(ChannelProperty::Name.flag()).unwrap(),
        value(&values, "CHANNEL_NAME")
    );
    assert_eq!(
        i64::try_from(ServerProperty::UniqueIdentifier.flag()).unwrap(),
        value(&values, "VIRTUALSERVER_UNIQUE_IDENTIFIER")
    );
    assert_eq!(
        i64::try_from(ServerProperty::Name.flag()).unwrap(),
        value(&values, "VIRTUALSERVER_NAME")
    );
}

#[test]
fn connect_status_and_visibility_read_the_header_values() {
    let values = enum_values();
    let statuses = [
        (ConnectStatus::Disconnected, "STATUS_DISCONNECTED"),
        (ConnectStatus::Connecting, "STATUS_CONNECTING"),
        (ConnectStatus::Connected, "STATUS_CONNECTED"),
        (
            ConnectStatus::ConnectionEstablishing,
            "STATUS_CONNECTION_ESTABLISHING",
        ),
        (
            ConnectStatus::ConnectionEstablished,
            "STATUS_CONNECTION_ESTABLISHED",
        ),
    ];
    for (status, enumerator) in statuses {
        let raw = i32::try_from(value(&values, enumerator)).unwrap();
        assert_eq!(ConnectStatus::from_raw(raw), status, "{enumerator}");
        assert_eq!(status.to_raw(), raw, "{enumerator}");
    }
    let visibilities = [
        (Visibility::Enter, "ENTER_VISIBILITY"),
        (Visibility::Retain, "RETAIN_VISIBILITY"),
        (Visibility::Leave, "LEAVE_VISIBILITY"),
    ];
    for (visibility, enumerator) in visibilities {
        let raw = i32::try_from(value(&values, enumerator)).unwrap();
        assert_eq!(Visibility::from_raw(raw), visibility, "{enumerator}");
    }
}

#[test]
fn talk_status_reads_and_writes_the_header_values() {
    let values = enum_values();
    let statuses = [
        (TalkStatus::NotTalking, "STATUS_NOT_TALKING"),
        (TalkStatus::Talking, "STATUS_TALKING"),
        (
            TalkStatus::TalkingWhileDisabled,
            "STATUS_TALKING_WHILE_DISABLED",
        ),
    ];
    for (status, enumerator) in statuses {
        let raw = i32::try_from(value(&values, enumerator)).unwrap();
        assert_eq!(TalkStatus::from_raw(raw), status, "{enumerator}");
        assert_eq!(status.to_raw(), raw, "{enumerator}");
    }
    assert_eq!(TalkStatus::from_raw(3), TalkStatus::Other(3));
    assert_eq!(TalkStatus::Other(3).to_raw(), 3);
}

#[test]
fn plugin_targets_log_levels_and_info_items_use_the_header_values() {
    let values = enum_values();
    let targets = [
        (
            PluginCommandTarget::CurrentChannel,
            "PluginCommandTarget_CURRENT_CHANNEL",
        ),
        (PluginCommandTarget::Server, "PluginCommandTarget_SERVER"),
        (
            PluginCommandTarget::Clients(&[]),
            "PluginCommandTarget_CLIENT",
        ),
        (
            PluginCommandTarget::CurrentChannelSubscribedClients,
            "PluginCommandTarget_CURRENT_CHANNEL_SUBSCRIBED_CLIENTS",
        ),
    ];
    for (target, enumerator) in targets {
        assert_eq!(
            i64::from(target.mode()),
            value(&values, enumerator),
            "{enumerator}"
        );
    }
    let levels = [
        (LogLevel::Critical, "LogLevel_CRITICAL"),
        (LogLevel::Error, "LogLevel_ERROR"),
        (LogLevel::Warning, "LogLevel_WARNING"),
        (LogLevel::Debug, "LogLevel_DEBUG"),
        (LogLevel::Info, "LogLevel_INFO"),
        (LogLevel::Devel, "LogLevel_DEVEL"),
    ];
    for (level, enumerator) in levels {
        assert_eq!(
            level.to_sdk() as i64,
            value(&values, enumerator),
            "{enumerator}"
        );
    }
    let server = i32::try_from(value(&values, "PLUGIN_SERVER")).unwrap();
    let channel = i32::try_from(value(&values, "PLUGIN_CHANNEL")).unwrap();
    let client = i32::try_from(value(&values, "PLUGIN_CLIENT")).unwrap();
    assert_eq!(InfoItem::from_raw(server, 0), Some(InfoItem::Server));
    assert_eq!(
        InfoItem::from_raw(channel, 8),
        Some(InfoItem::Channel(crate::ChannelId(8)))
    );
    assert_eq!(
        InfoItem::from_raw(client, 8),
        Some(InfoItem::Client(crate::ClientId(8)))
    );
    assert_eq!(
        i64::from(crate::export_glue::OFFERS_CONFIGURE),
        value(&values, "PLUGIN_OFFERS_NO_CONFIGURE")
    );
}

#[test]
fn the_header_parser_counts_implicit_and_explicit_values() {
    let values = enum_values();
    assert_eq!(value(&values, "CLIENT_META_DATA"), 15);
    assert_eq!(value(&values, "LogLevel_DEVEL"), 5);
    assert_eq!(value(&values, "PluginCommandTarget_CLIENT"), 2);
}
