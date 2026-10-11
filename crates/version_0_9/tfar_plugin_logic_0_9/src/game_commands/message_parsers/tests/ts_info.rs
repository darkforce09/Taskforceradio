//! Unit tests for `TS_INFO`, against `reference/tfar_0_9_12/ts/src/plugin.cpp:1773` and `ts_info`
//! (plugin.cpp:1534-1560).

use super::*;
use crate::game_commands::error::Error;

#[test]
fn ts_info_reads_each_query() {
    let parse = |message| GameCommand::parse(message).unwrap();
    assert_eq!(
        parse("TS_INFO\tSERVER"),
        GameCommand::TsInfo(TsInfoQuery::Server)
    );
    assert_eq!(
        parse("TS_INFO\tCHANNEL"),
        GameCommand::TsInfo(TsInfoQuery::Channel)
    );
    assert_eq!(
        parse("TS_INFO\tPING"),
        GameCommand::TsInfo(TsInfoQuery::Ping)
    );
    assert_eq!(
        parse("TS_INFO\tping"),
        GameCommand::TsInfo(TsInfoQuery::Other("ping".to_owned()))
    );
}

#[test]
fn ts_info_needs_exactly_two_fields() {
    for (message, found) in [("TS_INFO", 1), ("TS_INFO\tPING\tX", 3)] {
        assert_eq!(
            parse_ts_info(&tfar_foundation::arma_values::split_fields(message, '\t')).unwrap_err(),
            Error::WrongFieldCount {
                command: "TS_INFO",
                expected: 2,
                found
            }
        );
    }
}
