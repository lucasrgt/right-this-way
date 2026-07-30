mod common;

use common::*;
use rusqlite::Connection;
use std::{fs, path::Path, process::Command};

#[test]
fn csm_storage_is_opt_in_and_does_not_rewrite_root_files() {
    let temp = repo();
    let agents = "# Existing\n";
    fs::write(temp.path().join("AGENTS.md"), agents).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_rtw"))
        .current_dir(temp.path())
        .env("CSM_STORAGE_ROOT", ".csm")
        .arg("init")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(temp.path().join(".csm/rtw/config.local.toml").is_file());
    assert!(!temp.path().join(".rtw").exists());
    assert_eq!(fs::read_to_string(temp.path().join("AGENTS.md")).unwrap(), agents);
    assert!(!temp.path().join(".gitignore").exists());
}

#[test]
fn init_is_idempotent_and_manages_agent_files() {
    let temp = repo();
    fs::write(temp.path().join("AGENTS.md"), "# Existing\n").unwrap();
    rtw::init(temp.path(), &[Path::new("AGENTS.md").into(), Path::new("CLAUDE.md").into()]).unwrap();
    rtw::init(temp.path(), &[Path::new("AGENTS.md").into(), Path::new("CLAUDE.md").into()]).unwrap();
    let agents = fs::read_to_string(temp.path().join("AGENTS.md")).unwrap();
    assert!(agents.starts_with("# Existing"));
    assert_eq!(agents.matches("<!-- rtw:instructions:start -->").count(), 1);
    assert!(temp.path().join(".rtw/SKILL.md").is_file());
    assert!(temp.path().join(".rtw/config.local.toml").is_file());
    assert!(fs::read_to_string(temp.path().join(".gitignore")).unwrap().contains(".rtw/config.local.toml"));
    assert!(fs::read_to_string(temp.path().join(".gitignore")).unwrap().contains(".rtw/index.sqlite"));
    assert!(rtw::init(temp.path(), &[Path::new("../outside.md").into()]).is_err());
}

#[test]
fn add_persists_a_normalized_proven_way() {
    let temp = initialized();
    let mut input = input();
    input.tags.push("state".into());
    input.scopes.push(r"src\features\**".into());
    input.references = vec![r"src\features\orders\order_view_model.rs".into()];
    let way = rtw::add(temp.path(), input).unwrap();
    assert_eq!(way.recorded_by, "Pattern Author");
    assert_eq!(way.tags, ["state", "view-model"]);
    assert!(way.scopes.contains(&"src/features/**".into()));
    assert_eq!(way.references, ["src/features/orders/order_view_model.rs"]);
    assert_eq!(way.recorded_commit.len(), 40);
    let stored = fs::read_to_string(temp.path().join(format!(".rtw/ways/{}.toml", way.id))).unwrap();
    assert_eq!(toml::from_str::<rtw::Way>(&stored).unwrap(), way);
}

#[test]
fn add_rejects_unproven_or_unsafe_inputs() {
    let temp = initialized();
    let mut candidate = input();
    candidate.title = " ".into();
    assert!(rtw::add(temp.path(), candidate).is_err());
    let mut candidate = input();
    candidate.tags.clear();
    assert!(rtw::add(temp.path(), candidate).is_err());
    let mut candidate = input();
    candidate.scopes = vec!["[".into()];
    assert!(rtw::add(temp.path(), candidate).is_err());
    let mut candidate = input();
    candidate.references = vec!["../outside.rs".into()];
    assert!(rtw::add(temp.path(), candidate).is_err());
    let mut candidate = input();
    candidate.references = vec!["missing.rs".into()];
    assert!(rtw::add(temp.path(), candidate).is_err());
    fs::write(temp.path().join("untracked.rs"), "").unwrap();
    let mut candidate = input();
    candidate.references = vec!["untracked.rs".into()];
    assert!(rtw::add(temp.path(), candidate).is_err());
}

#[test]
fn guide_combines_scope_tags_and_full_text_across_directories() {
    let temp = initialized();
    let view_model = add_way(temp.path());
    let mut ui = input();
    ui.title = "Tokenized cards".into();
    ui.intent = "Build visual cards".into();
    ui.guidance = "Use semantic design tokens for every color and spacing value.".into();
    ui.tags = vec!["design-token".into(), "card".into()];
    ui.scopes = vec!["src/ui/**".into()];
    let card = rtw::add(temp.path(), ui).unwrap();

    let by_tag = rtw::guide(
        temp.path(),
        "Create a payment view-model for a new feature",
        &["src/features/payments/payment_view_model.rs".into()],
        8,
    )
    .unwrap();
    assert_eq!(by_tag[0].id, view_model.id);

    let by_text = rtw::guide(temp.path(), "Build a visual surface using semantic colors", &[], 8).unwrap();
    assert_eq!(by_text[0].id, card.id);

    let by_scope = rtw::guide(temp.path(), "Unrelated work", &["src/ui/payment_card.rs".into()], 1).unwrap();
    assert_eq!(by_scope[0].id, card.id);
    assert!(rtw::guide(temp.path(), "nothing relevant", &[], 8).unwrap().is_empty());
}

#[test]
fn guide_ranks_the_scoped_target_after_large_full_text_ties() {
    let temp = initialized();
    for index in 0..70 {
        let mut duplicate = input();
        duplicate.title = "Documented commands are executable".into();
        duplicate.intent = "Add a command to user documentation".into();
        duplicate.guidance = "Present a complete copyable command using current flags.".into();
        duplicate.tags = vec!["pattern15".into()];
        duplicate.scopes = vec![format!("docs/archive-{index}/**")];
        rtw::add(temp.path(), duplicate).unwrap();
    }
    let mut distractor = input();
    distractor.title = "Validate data boundaries".into();
    distractor.tags = vec!["data".into()];
    distractor.scopes = vec!["data/current/**".into()];
    rtw::add(temp.path(), distractor).unwrap();
    let mut target = input();
    target.title = "Documented commands are executable".into();
    target.intent = "Add a command to user documentation".into();
    target.guidance = "Present a complete copyable command using current flags.".into();
    target.tags = vec!["pattern15".into()];
    target.scopes = vec!["data/current/**".into()];
    let target = rtw::add(temp.path(), target).unwrap();

    let guided = rtw::guide(
        temp.path(),
        "Apply pattern15 for an executable command example",
        &["data/current/new/executable-command-example.txt".into()],
        8,
    )
    .unwrap();
    assert_eq!(guided[0].id, target.id);
}

#[test]
fn guide_rebuilds_disposable_index_and_validates_inputs() {
    let temp = initialized();
    let way = add_way(temp.path());
    fs::write(temp.path().join(".rtw/index.sqlite"), "corrupt").unwrap();
    assert!(!rtw::guide(temp.path(), "view model", &[], 8).unwrap().is_empty());
    let index = temp.path().join(".rtw/index.sqlite");
    Connection::open(&index)
        .unwrap()
        .execute_batch("CREATE TABLE sentinel(value INTEGER); INSERT INTO sentinel VALUES(7);")
        .unwrap();
    assert!(!rtw::guide(temp.path(), "view model", &[], 8).unwrap().is_empty());
    let sentinel: i64 = Connection::open(&index)
        .unwrap()
        .query_row("SELECT value FROM sentinel", [], |row| row.get(0))
        .unwrap();
    assert_eq!(sentinel, 7);
    let path = temp.path().join(format!(".rtw/ways/{}.toml", way.id));
    let mut changed: rtw::Way = toml::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    changed.guidance = "Preserve the fingerprintmarker convention.".into();
    fs::write(&path, toml::to_string_pretty(&changed).unwrap()).unwrap();
    assert_eq!(rtw::guide(temp.path(), "fingerprintmarker", &[], 8).unwrap()[0].id, way.id);
    assert!(
        Connection::open(index)
            .unwrap()
            .query_row("SELECT value FROM sentinel", [], |row| row.get::<_, i64>(0))
            .is_err()
    );
    assert!(rtw::guide(temp.path(), "x", &[], 8).unwrap().is_empty());
    assert!(rtw::guide(temp.path(), "", &[], 8).is_err());
    assert!(rtw::guide(temp.path(), "task", &[], 0).is_err());
    assert!(rtw::guide(temp.path(), "task", &[], 51).is_err());
    let raw = repo();
    assert!(rtw::guide(raw.path(), "task", &[], 8).is_err());
    assert!(rtw::repository(Path::new("Z:/definitely-not-a-repository")).is_err());
}

#[test]
fn corrupt_way_fails_closed() {
    let temp = initialized();
    fs::write(temp.path().join(".rtw/ways/bad.toml"), "not = [valid").unwrap();
    assert!(rtw::guide(temp.path(), "anything", &[], 8).is_err());
}
