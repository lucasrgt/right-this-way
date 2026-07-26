mod common;

use common::*;
use serde_json::{Value, json};
use std::{
    ffi::OsString,
    io::{self, Cursor, Write},
    process::Command,
};

struct FailWriter {
    writes: usize,
    fail_at: usize,
}

impl Write for FailWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.writes += 1;
        if self.writes >= self.fail_at {
            Err(io::Error::other("writer failed"))
        } else {
            Ok(bytes.len())
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

struct FailAfterLine(bool);

impl Write for FailAfterLine {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.0 {
            return Err(io::Error::other("writer failed after first line"));
        }
        if bytes.contains(&b'\n') {
            self.0 = true;
        }
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn cli(root: &std::path::Path, args: &[&str]) -> anyhow::Result<(i32, String)> {
    let mut arguments = vec!["rtw"];
    arguments.extend_from_slice(args);
    let mut output = Vec::new();
    let code = rtw::run_cli_at(os_args(&arguments), root, &mut Cursor::new(""), &mut output)?;
    Ok((code, String::from_utf8(output).unwrap()))
}

fn os_args(values: &[&str]) -> Vec<OsString> {
    values.iter().map(OsString::from).collect()
}

#[test]
fn cli_covers_init_add_guide_check_and_errors() {
    assert!(rtw::run_cli_env().is_err());
    let temp = repo();
    assert!(cli(temp.path(), &["init", "--agent-file", "CLAUDE.md"]).unwrap().1.contains("initialized"));
    let (_, added) = cli(
        temp.path(),
        &[
            "add",
            "--title",
            "Feature view models",
            "--intent",
            "Create workflow state",
            "--guidance",
            "Keep state in a view model.",
            "--scope",
            "src/features/**",
            "--tag",
            "view-model",
            "--reference",
            "src/features/orders/order_view_model.rs",
            "--json",
        ],
    )
    .unwrap();
    assert_eq!(serde_json::from_str::<Value>(&added).unwrap()["title"], "Feature view models");
    let (_, guided) = cli(
        temp.path(),
        &["guide", "--task", "new view model", "--path", "src/features/payments/vm.rs", "--json"],
    )
    .unwrap();
    assert_eq!(serde_json::from_str::<Vec<Value>>(&guided).unwrap().len(), 1);
    assert!(
        cli(temp.path(), &["guide", "--task", "new view model"])
            .unwrap()
            .1
            .contains("Feature view models")
    );
    assert!(cli(temp.path(), &["guide", "--task", "nothing"]).unwrap().1.contains("No relevant ways"));
    assert!(cli(temp.path(), &["check", "--task", "nothing"]).unwrap().1.contains("Aligned"));
    assert!(cli(temp.path(), &["guide"]).is_err());
}

#[test]
fn packaged_entrypoint_reaches_the_shared_cli() {
    let mut binary = std::env::current_exe().unwrap();
    binary.pop();
    binary.pop();
    binary.push(if cfg!(windows) { "rtw.exe" } else { "rtw" });
    let output = Command::new(binary).arg("--version").output().unwrap();
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap().trim(), format!("rtw {}", env!("CARGO_PKG_VERSION")));
}

#[test]
fn cli_check_uses_recurrence_and_failure_contracts() {
    let temp = initialized();
    let way = add_way(temp.path());
    change(temp.path());
    judge(temp.path(), "fail", &way);
    let (code, output) = cli(temp.path(), &["check", "--task", "Create a payment view model"]).unwrap();
    assert_eq!(code, 1);
    assert!(output.contains("Known pattern deviations"));
    assert!(output.contains("payment_view_model.rs"));
    judge(temp.path(), "exit", &way);
    assert!(cli(temp.path(), &["check", "--task", "Create a payment view model"]).is_err());
}

#[test]
fn cli_propagates_domain_and_output_failures() {
    let temp = initialized();
    let bad_add = [
        "rtw",
        "add",
        "--title",
        "Invalid",
        "--intent",
        "Invalid",
        "--guidance",
        "Invalid",
        "--scope",
        "**",
        "--tag",
        "invalid",
        "--reference",
        "missing.rs",
    ];
    assert!(rtw::run_cli_at(os_args(&bad_add), temp.path(), &mut Cursor::new(""), &mut Vec::new()).is_err());

    let way = add_way(temp.path());
    change(temp.path());
    judge(temp.path(), "fail", &way);
    for (args, fail_at) in [
        (vec!["rtw", "init"], 1),
        (vec!["rtw", "guide", "--task", "view model", "--json"], 1),
        (vec!["rtw", "guide", "--task", "view model"], 1),
        (vec!["rtw", "check", "--task", "Create a payment view model"], 1),
        (vec!["rtw", "check", "--task", "Create a payment view model"], 2),
        (
            vec![
                "rtw",
                "add",
                "--title",
                "Another pattern",
                "--intent",
                "Another intent",
                "--guidance",
                "Another guide",
                "--scope",
                "**",
                "--tag",
                "another",
                "--reference",
                "README.md",
            ],
            1,
        ),
    ] {
        assert!(rtw::run_cli_at(os_args(&args), temp.path(), &mut Cursor::new(""), &mut FailWriter { writes: 0, fail_at }).is_err());
    }
    assert!(
        rtw::run_cli_at(
            os_args(&["rtw", "check", "--task", "Create a payment view model"]),
            temp.path(),
            &mut Cursor::new(""),
            &mut FailAfterLine(false)
        )
        .is_err()
    );
}

#[test]
fn mcp_exposes_every_tool_and_calls_the_same_core() {
    let temp = initialized();
    let repository = temp.path().display().to_string();
    let requests = [
        json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}),
        json!({"jsonrpc":"2.0","method":"notifications/initialized","params":{}}),
        json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}),
        json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"rtw_add","arguments":{"repository":repository,"title":"Feature view models","intent":"Create workflow state","guidance":"Keep state in a view model.","scopes":["src/features/**"],"tags":["view-model"],"references":["src/features/orders/order_view_model.rs"]}}}),
        json!({"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"rtw_guide","arguments":{"repository":repository,"task":"new view model","paths":["src/features/payments/vm.rs"]}}}),
        json!({"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"rtw_check","arguments":{"repository":repository,"task":"nothing"}}}),
        json!({"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"unknown","arguments":{"repository":repository}}}),
        json!({"jsonrpc":"2.0","id":7,"method":"unknown","params":{}}),
        json!({"jsonrpc":"2.0","id":8,"method":"tools/call","params":{"name":"rtw_check","arguments":{"repository":repository}}}),
        json!({"jsonrpc":"2.0","id":9,"method":"tools/call","params":{"name":"rtw_add","arguments":{"repository":repository,"title":"Bad","intent":"Bad","guidance":"Bad","scopes":["**"],"tags":["bad"],"references":["missing.rs"]}}}),
        json!({"jsonrpc":"2.0","id":10,"method":"tools/call","params":{"name":"rtw_check","arguments":{"repository":repository,"task":"bad base","base":"--bad"}}}),
    ];
    let input = requests
        .iter()
        .map(|request| serde_json::to_string(request).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    let mut output = Vec::new();
    rtw::mcp_stream(&mut Cursor::new(input), &mut output).unwrap();
    let responses = String::from_utf8(output)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(responses.len(), 10);
    assert_eq!(responses[0]["result"]["serverInfo"]["name"], "right-this-way");
    assert_eq!(responses[1]["result"]["tools"].as_array().unwrap().len(), 3);
    assert!(responses[2]["result"]["content"][0]["text"].as_str().unwrap().contains("Feature view models"));
    assert!(responses[3]["result"]["content"][0]["text"].as_str().unwrap().contains("Feature view models"));
    assert_eq!(responses[4]["result"]["isError"], Value::Null);
    assert_eq!(responses[5]["result"]["isError"], true);
    assert_eq!(responses[6]["error"]["code"], -32601);
    assert_eq!(responses[9]["result"]["isError"], true);
    assert!(rtw::mcp_stream(&mut Cursor::new("not json\n"), &mut Vec::new()).is_err());
    assert!(
        rtw::mcp_stream(
            &mut Cursor::new("{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\"}\n"),
            &mut FailWriter { writes: 0, fail_at: 1 }
        )
        .is_err()
    );

    let mut cli_output = Vec::new();
    let request = "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/list\",\"params\":{}}\n";
    assert_eq!(
        rtw::run_cli_at(os_args(&["rtw", "mcp"]), temp.path(), &mut Cursor::new(request), &mut cli_output).unwrap(),
        0
    );
    assert!(String::from_utf8(cli_output).unwrap().contains("rtw_guide"));
}
