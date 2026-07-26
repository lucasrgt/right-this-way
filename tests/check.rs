mod common;

use common::*;
use std::fs;

#[test]
fn clean_or_irrelevant_diffs_short_circuit_without_a_judge() {
    let temp = initialized();
    let way = add_way(temp.path());
    assert_eq!(rtw::check(temp.path(), "create a view model", "HEAD").unwrap().ways_checked, 1);
    fs::write(temp.path().join("README.md"), "# Changed\n").unwrap();
    let result = rtw::check(temp.path(), "unrelated documentation", "HEAD").unwrap();
    assert_eq!(result.ways_checked, 0);
    assert!(result.deviations.is_empty());
    judge(temp.path(), "exit", &way);
    assert!(rtw::check(temp.path(), "unrelated documentation", "HEAD").is_ok());
}

#[test]
fn two_stage_judge_confirms_real_deviations() {
    let temp = initialized();
    let way = add_way(temp.path());
    change(temp.path());
    judge(temp.path(), "fail", &way);
    let result = rtw::check(temp.path(), "Create a payment view model", "HEAD").unwrap();
    assert_eq!(result.ways_checked, 1);
    assert_eq!(result.deviations.len(), 1);
    assert_eq!(result.deviations[0].way_id, way.id);
}

#[test]
fn second_stage_can_reject_a_first_stage_claim() {
    let temp = initialized();
    let way = add_way(temp.path());
    change(temp.path());
    judge(temp.path(), "reject", &way);
    assert!(rtw::check(temp.path(), "Create a payment view model", "HEAD").unwrap().deviations.is_empty());
}

#[test]
fn first_stage_can_pass_without_confirmation() {
    let temp = initialized();
    let way = add_way(temp.path());
    change(temp.path());
    judge(temp.path(), "pass", &way);
    assert!(rtw::check(temp.path(), "Create a payment view model", "HEAD").unwrap().deviations.is_empty());
}

#[test]
fn judge_and_protocol_failures_fail_closed() {
    for mode in ["exit", "invalid", "invent"] {
        let temp = initialized();
        let way = add_way(temp.path());
        change(temp.path());
        judge(temp.path(), mode, &way);
        assert!(rtw::check(temp.path(), "Create a payment view model", "HEAD").is_err(), "{mode}");
    }
}

#[test]
fn invalid_config_and_revisions_fail_closed() {
    let temp = initialized();
    add_way(temp.path());
    change(temp.path());
    assert!(rtw::check(temp.path(), "task", "--bad").is_err());
    assert!(rtw::check(temp.path(), "task", "missing").is_err());
    assert!(rtw::check(temp.path(), "", "HEAD").is_err());
    fs::write(temp.path().join(".rtw/config.local.toml"), "schema = 2\n[judge]\ncommand = []\n").unwrap();
    assert!(rtw::check(temp.path(), "Create a payment view model", "HEAD").is_err());
    fs::write(temp.path().join(".rtw/config.local.toml"), "bad toml").unwrap();
    assert!(rtw::check(temp.path(), "Create a payment view model", "HEAD").is_err());
    judge(temp.path(), "pass", &rtw::guide(temp.path(), "view model", &[], 1).unwrap()[0]);
    fs::rename(temp.path().join(".rtw/config.local.toml"), temp.path().join(".rtw/config.toml")).unwrap();
    assert!(rtw::check(temp.path(), "Create a payment view model", "HEAD").is_ok());
}

#[test]
fn oversized_diff_is_refused() {
    let temp = initialized();
    add_way(temp.path());
    fs::write(temp.path().join("README.md"), "x".repeat(121_000)).unwrap();
    assert!(rtw::check(temp.path(), "view model work", "HEAD").is_err());
}
