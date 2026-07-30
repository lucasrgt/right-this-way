use anyhow::{Context, Result, anyhow, bail};
use chrono::{DateTime, Utc};
use clap::{Args, Parser, Subcommand};
use glob::Pattern;
use rusqlite::{Connection, ToSql};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
#[rustfmt::skip]
use std::{collections::HashSet, ffi::OsString, fs, hash::{DefaultHasher, Hash, Hasher}, io::{self, BufRead, Write}, path::{Component, Path, PathBuf}, process::{Command, Stdio}};
use ulid::Ulid;

const SKILL: &str = include_str!("../assets/right-this-way/SKILL.md");
const CONFIG: &str = include_str!("../assets/config.toml");
const IGNORE: &str = include_str!("../assets/gitignore");
const INSTRUCTIONS: &str = include_str!("../assets/AGENT_INSTRUCTIONS.md");
const START: &str = "<!-- rtw:instructions:start -->";
const END: &str = "<!-- rtw:instructions:end -->";

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
#[rustfmt::skip]
pub struct Way { pub schema: u8, pub id: String, pub title: String, pub intent: String, pub guidance: String, pub scopes: Vec<String>, pub tags: Vec<String>, pub references: Vec<String>, pub recorded_at: DateTime<Utc>, pub recorded_by: String, pub recorded_commit: String }

#[derive(Clone, Debug)]
#[rustfmt::skip]
pub struct NewWay { pub title: String, pub intent: String, pub guidance: String, pub scopes: Vec<String>, pub tags: Vec<String>, pub references: Vec<String>, pub recorded_by: Option<String> }

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
#[rustfmt::skip]
pub struct Deviation { pub way_id: String, pub path: String, pub line: u64, pub reason: String }

#[derive(Clone, Debug, Serialize, PartialEq)]
#[rustfmt::skip]
pub struct CheckResult { pub ways_checked: usize, pub deviations: Vec<Deviation> }

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
#[rustfmt::skip]
struct Config { schema: u8, judge: Judge }

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
#[rustfmt::skip]
struct Judge { command: Vec<String> }

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
#[rustfmt::skip]
struct Audit { deviations: Vec<Deviation> }

pub fn repository(path: &Path) -> Result<PathBuf> {
    let root = git(path, &["rev-parse", "--show-toplevel"])?;
    fs::canonicalize(root.trim()).context("resolve repository root")
}

#[rustfmt::skip]
fn data_dir(root: &Path) -> PathBuf { match std::env::var_os("CSM_STORAGE_ROOT") { Some(path) => { let path = PathBuf::from(path); let path = if path.is_absolute() { path } else { root.join(path) }; path.join("rtw") }, None => root.join(".rtw") } }

#[rustfmt::skip]
fn store_exclude(root: &Path) -> String { let relative = data_dir(root).strip_prefix(root).ok().map(|path| path.to_string_lossy().replace('\\', "/")).unwrap_or_else(|| "__csm_external_store__".into()); format!(":(exclude){relative}/**") }

#[rustfmt::skip]
pub fn init(root: &Path, agent_files: &[PathBuf]) -> Result<()> { let root = repository(root)?; let data = data_dir(&root); fs::create_dir_all(data.join("ways"))?; write_new(data.join("config.local.toml"), CONFIG)?; fs::write(data.join("SKILL.md"), SKILL)?; if std::env::var_os("CSM_STORAGE_ROOT").is_none() { append_once(root.join(".gitignore"), IGNORE)?; for file in agent_files { safe_relative(file)?; upsert_block(root.join(file), INSTRUCTIONS)?; } } Ok(()) }

pub fn add(root: &Path, input: NewWay) -> Result<Way> {
    let root = repository(root)?;
    require_text("title", &input.title)?;
    require_text("intent", &input.intent)?;
    require_text("guidance", &input.guidance)?;
    let scopes = normalized(input.scopes);
    let tags = normalized(input.tags);
    let references = normalized(input.references);
    if scopes.is_empty() || tags.is_empty() || references.is_empty() {
        bail!("a way requires at least one scope, tag, and reference")
    }
    for scope in &scopes {
        Pattern::new(scope).with_context(|| format!("invalid scope {scope}"))?;
    }
    for reference in &references {
        let relative = Path::new(reference);
        safe_relative(relative)?;
        if !root.join(relative).is_file() {
            bail!("reference does not exist: {reference}")
        }
        git(&root, &["ls-files", "--error-unmatch", reference]).with_context(|| format!("reference is not tracked: {reference}"))?;
    }
    let way = Way {
        schema: 1,
        id: Ulid::generate().to_string().to_lowercase(),
        title: input.title.trim().into(),
        intent: input.intent.trim().into(),
        guidance: input.guidance.trim().into(),
        scopes,
        tags,
        references,
        recorded_at: Utc::now(),
        recorded_by: input
            .recorded_by
            .unwrap_or(git(&root, &["config", "user.name"]).unwrap_or_else(|_| "unknown".into()))
            .trim()
            .into(),
        recorded_commit: git(&root, &["rev-parse", "HEAD"])?.trim().into(),
    };
    let directory = data_dir(&root).join("ways");
    fs::create_dir_all(&directory)?;
    write_new(directory.join(format!("{}.toml", way.id)), &toml::to_string_pretty(&way)?)?;
    invalidate(&root);
    Ok(way)
}

pub fn guide(root: &Path, task: &str, paths: &[String], limit: usize) -> Result<Vec<Way>> {
    let root = repository(root)?;
    require_text("task", task)?;
    if limit == 0 || limit > 50 {
        bail!("limit must be between 1 and 50")
    }
    let ways = load_ways(&root)?;
    let connection = rebuild_index(&root, &ways)?;
    let terms = tokens(&format!("{task} {}", paths.join(" ")));
    let matches: HashSet<_> = fts_ids(&connection, &terms)?.into_iter().collect();
    let mut scored = ways
        .into_iter()
        .filter_map(|way| {
            let scope = paths.iter().any(|path| {
                way.scopes
                    .iter()
                    .any(|scope| Pattern::new(scope).is_ok_and(|pattern| pattern.matches(&path.replace('\\', "/"))))
            });
            let tags = way.tags.iter().filter(|tag| terms.contains(&tag.to_lowercase())).count();
            let relevance = way_relevance(&matches, &way, &terms);
            let score = usize::from(scope) * 1_000 + tags * 100 + relevance;
            (score > 0).then_some((score, way))
        })
        .collect::<Vec<_>>();
    scored.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.title.cmp(&right.1.title)));
    Ok(scored.into_iter().take(limit).map(|(_, way)| way).collect())
}

pub fn check(root: &Path, task: &str, base: &str) -> Result<CheckResult> {
    let root = repository(root)?;
    require_text("task", task)?;
    validate_revision(base)?;
    git(&root, &["rev-parse", "--verify", base])?;
    let exclude = store_exclude(&root);
    let mut paths = git(&root, &["diff", "--name-only", base, "--", ".", &exclude])?
        .lines()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let untracked = git(&root, &["ls-files", "--others", "--exclude-standard", "--", ".", &exclude])?;
    paths.extend(untracked.lines().map(str::to_owned));
    let ways = guide(&root, task, &paths, 12)?;
    if ways.is_empty() {
        return Ok(clean_check(0));
    }
    let mut diff = git(&root, &["diff", "--no-ext-diff", "--unified=3", base, "--", ".", &exclude])?;
    for path in untracked.lines() {
        let contents = fs::read_to_string(root.join(path)).with_context(|| format!("untracked file is not auditable text: {path}"))?;
        diff.push_str(&format!(
            "\ndiff --git a/{path} b/{path}\nnew file mode 100644\n--- /dev/null\n+++ b/{path}\n@@ -0,0 +1,{} @@\n",
            contents.lines().count()
        ));
        for line in contents.lines() {
            diff.push_str(&format!("+{line}\n"));
        }
    }
    if diff.len() > 120_000 {
        bail!("diff exceeds the 120000 byte audit limit")
    }
    if diff.trim().is_empty() {
        return Ok(clean_check(ways.len()));
    }
    let first = judge(&root, &audit_prompt(task, &ways, &diff, None)?)?;
    let candidates = valid_deviations(first.deviations, &ways, &paths)?;
    if candidates.is_empty() {
        return Ok(clean_check(ways.len()));
    }
    let second = judge(&root, &audit_prompt(task, &ways, &diff, Some(&candidates))?)?;
    let confirmed = valid_deviations(second.deviations, &ways, &paths)?;
    let keys = candidates.iter().map(deviation_key).collect::<HashSet<_>>();
    Ok(CheckResult {
        ways_checked: ways.len(),
        deviations: confirmed.into_iter().filter(|item| keys.contains(&deviation_key(item))).collect(),
    })
}

fn load_ways(root: &Path) -> Result<Vec<Way>> {
    let directory = data_dir(root).join("ways");
    if !directory.exists() {
        bail!("Right This Way is not initialized; run rtw init")
    }
    let mut paths = fs::read_dir(directory)?
        .map(|entry| entry.map(|value| value.path()))
        .collect::<std::result::Result<Vec<_>, _>>()?;
    paths.sort();
    paths
        .into_iter()
        .filter(|path| path.extension().is_some_and(|extension| extension == "toml"))
        .map(|path| toml::from_str(&fs::read_to_string(&path)?).with_context(|| format!("invalid way {}", path.display())))
        .collect()
}

#[rustfmt::skip]
fn rebuild_index(root: &Path, ways: &[Way]) -> Result<Connection> {
    let path = data_dir(root).join("index.sqlite");
    let fingerprint = { let mut hasher = DefaultHasher::new(); serde_json::to_string(ways)?.hash(&mut hasher); format!("{:x}", hasher.finish()) };
    if let Ok(connection) = Connection::open(&path) { let cached = connection.query_row("SELECT fingerprint FROM metadata LIMIT 1", [], |row| row.get::<_, String>(0)).ok(); if cached.as_deref() == Some(&fingerprint) { return Ok(connection); } drop(connection); }
    invalidate(root);
    let mut connection = Connection::open(path)?;
    connection.execute_batch("CREATE TABLE metadata(fingerprint TEXT NOT NULL); CREATE VIRTUAL TABLE search USING fts5(id UNINDEXED,title,intent,guidance,tags,tokenize='porter unicode61');")?;
    let transaction = connection.transaction()?; transaction.execute("INSERT INTO metadata VALUES (?1)", [&fingerprint])?;
    for way in ways { let tags = way.tags.join(" "); let values: [&dyn ToSql; 5] = [&way.id, &way.title, &way.intent, &way.guidance, &tags]; transaction.execute("INSERT INTO search(id,title,intent,guidance,tags) VALUES (?1,?2,?3,?4,?5)", values)?; }
    transaction.commit()?; Ok(connection)
}

fn fts_ids(connection: &Connection, terms: &HashSet<String>) -> Result<Vec<String>> {
    if terms.is_empty() {
        return Ok(Vec::new());
    }
    let query = terms.iter().map(|term| format!("\"{term}\"")).collect::<Vec<_>>().join(" OR ");
    let mut statement = connection.prepare("SELECT id FROM search WHERE search MATCH ?1 ORDER BY bm25(search)")?;
    Ok(statement.query_map([query], |row| row.get(0))?.collect::<std::result::Result<Vec<_>, _>>()?)
}

fn audit_prompt(task: &str, ways: &[Way], diff: &str, candidates: Option<&[Deviation]>) -> Result<String> {
    let phase = if let Some(items) = candidates {
        format!(
            "Confirm only these claimed deviations and reject unsupported ones: {}.",
            serde_json::to_string(items)?
        )
    } else {
        "Find concrete deviations from the supplied ways.".into()
    };
    Ok(format!(
        "You are an isolated repository-pattern auditor. {phase} Use only the supplied ways and diff. Do not invent requirements. Return strict JSON {{\"deviations\":[{{\"way_id\":\"id\",\"path\":\"changed/file\",\"line\":1,\"reason\":\"specific mismatch\"}}]}}. Task: {task}\nWAYS:\n{}\nDIFF:\n{diff}",
        serde_json::to_string(ways)?
    ))
}

fn judge(root: &Path, prompt: &str) -> Result<Audit> {
    let config = load_config(root)?;
    if config.schema != 1 || config.judge.command.is_empty() {
        bail!("unsupported or empty judge configuration")
    }
    let mut child = Command::new(&config.judge.command[0])
        .args(&config.judge.command[1..])
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .context("start judge")?;
    child.stdin.take().context("open judge stdin")?.write_all(prompt.as_bytes())?;
    let output = child.wait_with_output()?;
    if !output.status.success() {
        bail!("judge exited with {}", output.status)
    }
    serde_json::from_slice(&output.stdout).context("judge returned invalid JSON")
}

fn load_config(root: &Path) -> Result<Config> {
    let local = data_dir(root).join("config.local.toml");
    let project = data_dir(root).join("config.toml");
    let user = dirs::config_dir().map(|path| path.join("right-this-way/config.toml"));
    let path = [Some(local), Some(project), user]
        .into_iter()
        .flatten()
        .find(|path| path.exists())
        .ok_or_else(|| anyhow!("missing judge configuration"))?;
    toml::from_str(&fs::read_to_string(path)?).context("invalid judge configuration")
}

fn valid_deviations(items: Vec<Deviation>, ways: &[Way], paths: &[String]) -> Result<Vec<Deviation>> {
    let ids = ways.iter().map(|way| way.id.as_str()).collect::<HashSet<_>>();
    let changed = paths.iter().map(String::as_str).collect::<HashSet<_>>();
    if items
        .iter()
        .any(|item| !ids.contains(item.way_id.as_str()) || !changed.contains(item.path.as_str()) || item.line == 0 || item.reason.trim().is_empty())
    {
        bail!("judge returned an invented or incomplete deviation")
    }
    Ok(items)
}

#[rustfmt::skip]
fn clean_check(ways_checked: usize) -> CheckResult { CheckResult { ways_checked, deviations: Vec::new() } }

#[rustfmt::skip]
fn git(root: &Path, arguments: &[&str]) -> Result<String> {
    let output = Command::new("git").args(["-c", "core.quotePath=false"]).args(arguments).current_dir(root).output().context("start git")?;
    if !output.status.success() {
        bail!("git {} failed: {}", arguments.join(" "), String::from_utf8_lossy(&output.stderr).trim())
    }
    String::from_utf8(output.stdout).context("git output was not UTF-8")
}

#[rustfmt::skip]
fn safe_relative(path: &Path) -> Result<()> { if path.as_os_str().is_empty() || path.is_absolute() || path.components().any(|part| matches!(part, Component::ParentDir | Component::RootDir | Component::Prefix(_))) { bail!("path must stay inside the repository: {}", path.display()) } Ok(()) }

#[rustfmt::skip]
fn require_text(name: &str, value: &str) -> Result<()> { if value.trim().is_empty() { bail!("{name} must not be empty") } else { Ok(()) } }

#[rustfmt::skip]
fn normalized(values: Vec<String>) -> Vec<String> { let mut values = values.into_iter().map(|value| value.trim().replace('\\', "/")).filter(|value| !value.is_empty()).collect::<Vec<_>>(); values.sort(); values.dedup(); values }

#[rustfmt::skip]
fn tokens(value: &str) -> HashSet<String> { value.to_lowercase().split(|character: char| !character.is_alphanumeric()).filter(|part| part.len() > 1).map(str::to_owned).collect() }

#[rustfmt::skip]
fn way_terms(way: &Way) -> HashSet<String> { tokens(&format!("{} {} {} {}", way.title, way.intent, way.guidance, way.tags.join(" "))) }

#[rustfmt::skip]
fn way_relevance(matches: &HashSet<String>, way: &Way, terms: &HashSet<String>) -> usize { if matches.contains(&way.id) { way_terms(way).intersection(terms).count() } else { 0 } }

#[rustfmt::skip]
fn validate_revision(revision: &str) -> Result<()> { if revision.is_empty() || revision.starts_with('-') || !revision.chars().all(|character| character.is_ascii_alphanumeric() || "_./~^-".contains(character)) { bail!("invalid base revision") } Ok(()) }

#[rustfmt::skip]
fn deviation_key(item: &Deviation) -> (&str, &str, u64) { (&item.way_id, &item.path, item.line) }

#[rustfmt::skip]
fn write_new(path: PathBuf, contents: &str) -> Result<()> { if !path.exists() { fs::write(path, contents)?; } Ok(()) }

#[rustfmt::skip]
fn append_once(path: PathBuf, block: &str) -> Result<()> {
    let current = fs::read_to_string(&path).unwrap_or_default();
    if !current.contains(block.trim()) {
        fs::write(path, format!("{}{}{}\n", current, if current.is_empty() || current.ends_with('\n') { "" } else { "\n" }, block.trim()))?;
    }
    Ok(())
}

#[rustfmt::skip]
fn upsert_block(path: PathBuf, block: &str) -> Result<()> {
    let current = fs::read_to_string(&path).unwrap_or_default();
    let updated = if let (Some(start), Some(end)) = (current.find(START), current.find(END)) {
        format!("{}{}{}", &current[..start], block.trim(), &current[end + END.len()..])
    } else {
        format!("{}{}{}\n", current, if current.is_empty() || current.ends_with('\n') { "" } else { "\n" }, block.trim())
    };
    fs::write(path, updated)?;
    Ok(())
}

fn invalidate(root: &Path) {
    let _ = fs::remove_file(data_dir(root).join("index.sqlite"));
}

#[derive(Parser)]
#[command(name = "rtw", version, about = "Follow proven repository patterns")]
#[rustfmt::skip]
struct Cli { #[command(subcommand)] command: Commands }

#[derive(Subcommand)]
#[rustfmt::skip]
enum Commands { Init(InitArgs), Add(AddArgs), Guide(QueryArgs), Check(CheckArgs), Mcp }

#[derive(Args)]
#[rustfmt::skip]
struct InitArgs { #[arg(long, default_value = "AGENTS.md")] agent_file: Vec<PathBuf> }

#[derive(Args)]
#[rustfmt::skip]
struct AddArgs { #[arg(long)] title: String, #[arg(long)] intent: String, #[arg(long)] guidance: String, #[arg(long, required = true)] scope: Vec<String>, #[arg(long, required = true)] tag: Vec<String>, #[arg(long, required = true)] reference: Vec<String>, #[arg(long)] recorded_by: Option<String>, #[arg(long)] json: bool }

#[derive(Args)]
#[rustfmt::skip]
struct QueryArgs { #[arg(long)] task: String, #[arg(long)] path: Vec<String>, #[arg(long, default_value_t = 8)] limit: usize, #[arg(long)] json: bool }

#[derive(Args)]
#[rustfmt::skip]
struct CheckArgs { #[arg(long)] task: String, #[arg(long, default_value = "HEAD")] base: String, #[arg(long)] json: bool }

pub fn run_cli_env() -> Result<i32> {
    let current = std::env::current_dir()?;
    run_cli_at(std::env::args_os().collect(), &current, &mut io::stdin().lock(), &mut io::stdout())
}

#[rustfmt::skip]
pub fn run_cli_at(arguments: Vec<OsString>, current: &Path, input: &mut dyn BufRead, output: &mut dyn Write) -> Result<i32> {
    let cli = match Cli::try_parse_from(arguments) { Ok(cli) => cli, Err(error) if error.use_stderr() => return Err(error.into()), Err(error) => { write!(output, "{error}")?; return Ok(0) } };
    match cli.command {
        Commands::Init(args) => {
            init(current, &args.agent_file)?;
            writeln!(output, "Right This Way initialized.")?;
        }
        Commands::Add(args) => {
            let way = add(
                current,
                NewWay {
                    title: args.title,
                    intent: args.intent,
                    guidance: args.guidance,
                    scopes: args.scope,
                    tags: args.tag,
                    references: args.reference,
                    recorded_by: args.recorded_by,
                },
            )?;
            print_value(&way, args.json, output)?;
        }
        Commands::Guide(args) => print_value(&guide(current, &args.task, &args.path, args.limit)?, args.json, output)?,
        Commands::Check(args) => {
            let result = check(current, &args.task, &args.base)?;
            print_value(&result, args.json, output)?;
            return Ok(i32::from(!result.deviations.is_empty()));
        }
        Commands::Mcp => mcp_stream(input, output)?,
    }
    Ok(0)
}

fn print_value<T: Serialize>(value: &T, json_output: bool, output: &mut dyn Write) -> Result<()> {
    if json_output {
        writeln!(output, "{}", serde_json::to_string_pretty(value)?)?;
    } else {
        let value = serde_json::to_value(value)?;
        if let Some(ways) = value.as_array() {
            if ways.is_empty() {
                writeln!(output, "No relevant ways found.")?;
            }
            for way in ways {
                writeln!(
                    output,
                    "> {} [{}]\n  {}\n  References: {}",
                    way["title"].as_str().unwrap_or("Way"),
                    way["tags"]
                        .as_array()
                        .map(|items| items.iter().filter_map(Value::as_str).collect::<Vec<_>>().join(", "))
                        .unwrap_or_default(),
                    way["guidance"].as_str().unwrap_or(""),
                    way["references"]
                        .as_array()
                        .map(|items| items.iter().filter_map(Value::as_str).collect::<Vec<_>>().join(", "))
                        .unwrap_or_default()
                )?;
            }
        } else if value.get("deviations").is_some() {
            let deviations = value["deviations"].as_array().context("invalid check result")?;
            writeln!(
                output,
                "{}",
                if deviations.is_empty() {
                    "Aligned with relevant ways."
                } else {
                    "Known pattern deviations found."
                }
            )?;
            for item in deviations {
                writeln!(
                    output,
                    "x {}:{} {}",
                    item["path"].as_str().unwrap_or(""),
                    item["line"],
                    item["reason"].as_str().unwrap_or("")
                )?;
            }
        } else {
            writeln!(output, "{}", serde_json::to_string_pretty(&value)?)?;
        }
    }
    Ok(())
}

pub fn mcp_stream(reader: &mut dyn BufRead, output: &mut dyn Write) -> Result<()> {
    for line in reader.lines() {
        let request: Value = serde_json::from_str(&line?)?;
        if request.get("id").is_none() {
            continue;
        }
        let id = request["id"].clone();
        let response = match request["method"].as_str().unwrap_or("") {
            "initialize" => {
                json!({"protocolVersion":"2025-06-18","capabilities":{"tools":{}},"serverInfo":{"name":"right-this-way","version":env!("CARGO_PKG_VERSION")}})
            }
            "tools/list" => json!({"tools":[
                {"name":"rtw_guide","description":"Find relevant proven repository patterns","inputSchema":{"type":"object","required":["repository","task"],"properties":{"repository":{"type":"string"},"task":{"type":"string"},"paths":{"type":"array","items":{"type":"string"}}}}},
                {"name":"rtw_add","description":"Record a proven repository pattern","inputSchema":{"type":"object","required":["repository","title","intent","guidance","scopes","tags","references"],"properties":{"repository":{"type":"string"},"title":{"type":"string"},"intent":{"type":"string"},"guidance":{"type":"string"},"scopes":{"type":"array","items":{"type":"string"}},"tags":{"type":"array","items":{"type":"string"}},"references":{"type":"array","items":{"type":"string"}}}}},
                {"name":"rtw_check","description":"Audit a diff against relevant proven patterns","inputSchema":{"type":"object","required":["repository","task"],"properties":{"repository":{"type":"string"},"task":{"type":"string"},"base":{"type":"string"}}}}
            ]}),
            "tools/call" => match mcp_call(&request["params"]) {
                Ok(value) => json!({"content":[{"type":"text","text":serde_json::to_string(&value)?}]}),
                Err(error) => json!({"content":[{"type":"text","text":format!("{error:#}")}],"isError":true}),
            },
            method => json!({"code":-32601,"message":format!("unknown method {method}")}),
        };
        let envelope = if response.get("code").is_some() {
            json!({"jsonrpc":"2.0","id":id,"error":response})
        } else {
            json!({"jsonrpc":"2.0","id":id,"result":response})
        };
        writeln!(output, "{}", serde_json::to_string(&envelope)?)?;
    }
    Ok(())
}

fn mcp_call(parameters: &Value) -> Result<Value> {
    let name = parameters["name"].as_str().context("missing tool name")?;
    let args = &parameters["arguments"];
    let root = Path::new(args["repository"].as_str().context("missing repository")?);
    match name {
        "rtw_guide" => Ok(serde_json::to_value(guide(root, text(args, "task")?, &strings(args, "paths"), 8)?)?),
        "rtw_check" => Ok(serde_json::to_value(check(
            root,
            text(args, "task")?,
            args["base"].as_str().unwrap_or("HEAD"),
        )?)?),
        "rtw_add" => Ok(serde_json::to_value(add(
            root,
            NewWay {
                title: text(args, "title")?.into(),
                intent: text(args, "intent")?.into(),
                guidance: text(args, "guidance")?.into(),
                scopes: strings(args, "scopes"),
                tags: strings(args, "tags"),
                references: strings(args, "references"),
                recorded_by: None,
            },
        )?)?),
        _ => bail!("unknown tool {name}"),
    }
}

fn text<'a>(value: &'a Value, name: &str) -> Result<&'a str> {
    value[name].as_str().with_context(|| format!("missing {name}"))
}

fn strings(value: &Value, name: &str) -> Vec<String> {
    value[name]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect()
}
