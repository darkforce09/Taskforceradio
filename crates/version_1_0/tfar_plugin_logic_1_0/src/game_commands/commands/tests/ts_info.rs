//! Unit tests for `TsInfoQuery`, against `ts_info` (`reference/tfar_1_0/ts/src/CommandProcessor.cpp:615-629`).

use super::*;

#[test]
fn each_query_matches_its_exact_name() {
    assert_eq!(TsInfoQuery::parse("SERVER"), TsInfoQuery::Server);
    assert_eq!(TsInfoQuery::parse("SERVERUID"), TsInfoQuery::ServerUid);
    assert_eq!(TsInfoQuery::parse("CHANNEL"), TsInfoQuery::Channel);
    assert_eq!(TsInfoQuery::parse("CHANNELID"), TsInfoQuery::ChannelId);
    assert_eq!(TsInfoQuery::parse("PING"), TsInfoQuery::Ping);
    assert_eq!(TsInfoQuery::parse("VERSION"), TsInfoQuery::Version);
    assert_eq!(
        TsInfoQuery::parse("ping"),
        TsInfoQuery::Other("ping".to_owned())
    );
}

#[test]
fn ping_is_pong_and_an_unknown_query_is_fail() {
    // CommandProcessor.cpp:624-628.
    assert_eq!(TsInfoQuery::Ping.fixed_reply(), Some("PONG"));
    assert_eq!(
        TsInfoQuery::Other(String::new()).fixed_reply(),
        Some("FAIL")
    );
    assert_eq!(TsInfoQuery::Server.fixed_reply(), None);
    assert_eq!(TsInfoQuery::Version.fixed_reply(), None);
}
