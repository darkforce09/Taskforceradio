//! Unit tests for the dump folder names.

use super::*;

#[test]
fn the_epoch_is_the_first_of_january_at_midnight() {
    assert_eq!(dump_folder_name(0), "01-01_00-00-00");
    assert_eq!(civil_from_days(0), (1970, 1, 1));
}

#[test]
fn a_known_moment_gets_its_day_month_and_time() {
    // 2026-10-03 14:05:09 UTC.
    assert_eq!(dump_folder_name(1_791_036_309), "03-10_14-05-09");
}

#[test]
fn the_leap_day_is_a_day_of_its_own() {
    // 2024-02-29 and the day after.
    assert_eq!(civil_from_days(19_782), (2024, 2, 29));
    assert_eq!(civil_from_days(19_783), (2024, 3, 1));
}

#[test]
fn the_folder_lies_under_logs_of_the_config_path() {
    let folder = dump_folder(Path::new("/config"));
    assert!(folder.starts_with("/config/logs"));
}
