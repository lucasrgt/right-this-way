#![allow(dead_code)]

use rtw::{NewWay, Way};
use std::{fs, path::Path, process::Command};
use tempfile::TempDir;

pub fn git(root: &Path, args: &[&str]) -> String {
    let output = Command::new("git").args(args).current_dir(root).output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    String::from_utf8(output.stdout).unwrap()
}

pub fn repo() -> TempDir {
    let temp = tempfile::tempdir().unwrap();
    git(temp.path(), &["init", "-b", "main"]);
    git(temp.path(), &["config", "user.name", "Pattern Author"]);
    git(temp.path(), &["config", "user.email", "author@example.test"]);
    fs::create_dir_all(temp.path().join("src/features/orders")).unwrap();
    fs::write(temp.path().join("src/features/orders/order_view_model.rs"), "pub struct OrderViewModel;\n").unwrap();
    fs::write(temp.path().join("README.md"), "# Fixture\n").unwrap();
    git(temp.path(), &["add", "."]);
    git(temp.path(), &["commit", "-m", "initial"]);
    temp
}

pub fn input() -> NewWay {
    NewWay {
        title: "Feature view models".into(),
        intent: "Create a view model for a feature workflow".into(),
        guidance: "Keep state transitions in the view model and expose immutable state to the view.".into(),
        scopes: vec!["src/features/**".into()],
        tags: vec!["view-model".into(), "state".into()],
        references: vec!["src/features/orders/order_view_model.rs".into()],
        recorded_by: None,
    }
}

pub fn initialized() -> TempDir {
    let temp = repo();
    rtw::init(temp.path(), &[Path::new("AGENTS.md").into()]).unwrap();
    git(temp.path(), &["add", "."]);
    git(temp.path(), &["commit", "-m", "adopt rtw"]);
    temp
}

pub fn add_way(root: &Path) -> Way {
    let way = rtw::add(root, input()).unwrap();
    git(root, &["add", ".rtw/ways"]);
    git(root, &["commit", "-m", "record view model way"]);
    way
}

pub fn judge(root: &Path, mode: &str, way: &Way) {
    let script = root.join("judge.py");
    fs::write(
        &script,
        r#"import json,sys
mode,way_id,path=sys.argv[1:4]
prompt=sys.stdin.read()
if mode=="exit": sys.exit(7)
if mode=="invalid": print("not json"); sys.exit(0)
if mode=="invent": print(json.dumps({"deviations":[{"way_id":"invented","path":path,"line":1,"reason":"invented"}]})); sys.exit(0)
items=[]
if mode=="fail" or (mode=="reject" and "Confirm only" not in prompt):
    items=[{"way_id":way_id,"path":path,"line":1,"reason":"The view owns mutable workflow state instead of using the proven view-model boundary."}]
print(json.dumps({"deviations":items}))
"#,
    )
    .unwrap();
    let command = serde_json::to_string(&vec![
        "python".to_string(),
        script.display().to_string(),
        mode.into(),
        way.id.clone(),
        "src/features/payments/payment_view_model.rs".into(),
    ])
    .unwrap();
    fs::write(root.join(".rtw/config.local.toml"), format!("schema = 1\n[judge]\ncommand = {command}\n")).unwrap();
}

pub fn change(root: &Path) {
    fs::create_dir_all(root.join("src/features/payments")).unwrap();
    fs::write(
        root.join("src/features/payments/payment_view_model.rs"),
        "pub struct PaymentView { pub mutable_state: String }\n",
    )
    .unwrap();
}
