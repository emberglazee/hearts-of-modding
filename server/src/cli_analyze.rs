//! Static analysis without a transport: `hom-lsp analyze <file>`.
//!
//! The editor and this route share everything after the wire — the same
//! scanners, the same rule set, the same `publishDiagnostics` payload — because
//! they are the same `Backend`. What changes is only where the messages go: an
//! in-process socket instead of stdio. A script gets one command and one JSON
//! object, instead of JSON-RPC framing, an initialize ordering that silently
//! yields an empty index when it is wrong, and a scan-complete handshake.
//!
//! Exit codes: 0 clean, 1 diagnostics found, 2 the file could not be analysed.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use futures::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tower_lsp_server::LspService;
use tower_lsp_server::jsonrpc::{Id, Request, Response};
use tower_lsp_server::ls_types::Uri;
use tower_service::Service;

use crate::backend::Backend;

pub(crate) struct Options {
    pub(crate) path: PathBuf,
    pub(crate) game: Option<PathBuf>,
    /// Fall back to the usual Steam locations when `game` is `None`.
    pub(crate) discover_game: bool,
    pub(crate) workspaces: Vec<PathBuf>,
    pub(crate) scope_validation: bool,
    pub(crate) styling: bool,
    pub(crate) tokens: bool,
    pub(crate) json: bool,
    pub(crate) timeout: Duration,
}

pub(crate) async fn run(opts: Options) -> i32 {
    let path = match opts.path.canonicalize() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("analyze: {}: {e}", opts.path.display());
            return 2;
        }
    };
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("analyze: {}: {e}", path.display());
            return 2;
        }
    };
    let workspaces: Vec<PathBuf> = if opts.workspaces.is_empty() {
        vec![infer_workspace(&path)]
    } else {
        opts.workspaces
            .iter()
            .filter_map(|w| w.canonicalize().ok())
            .collect()
    };
    let Some(uri) = Uri::from_file_path(&path) else {
        eprintln!("analyze: {} is not a file path", path.display());
        return 2;
    };
    let uri_str = uri.to_string();

    let (mut service, socket) = LspService::new(Backend::new);
    let (mut outbound, mut sink) = socket.split();
    // Two reasons this task exists. The socket's queue is bounded, so a handler
    // that logs while it works blocks on the send unless somebody drains it.
    // And `initialized` *awaits* its `client/registerCapability` request after
    // the scan, so a client that only listens waits forever. This answers every
    // server-to-client request and forwards the rest into a queue we read at
    // our own pace.
    let folders = folders_json(&workspaces);
    let client_folders = folders.clone();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    tokio::spawn(async move {
        while let Some(req) = outbound.next().await {
            if let Some(id) = req.id() {
                let result = match req.method() {
                    // The extension answers these from settings; an empty
                    // object per item keeps the server's defaults.
                    "workspace/configuration" => Value::Array(vec![
                        json!({});
                        req.params()
                            .and_then(|p| p.get("items"))
                            .and_then(Value::as_array)
                            .map_or(0, Vec::len)
                    ]),
                    "workspace/workspaceFolders" => client_folders.clone(),
                    _ => Value::Null,
                };
                let _ = sink
                    .send(Response::from_parts(id.clone(), Ok(result)))
                    .await;
            }
            if tx.send(req).is_err() {
                break;
            }
        }
    });

    let game = opts
        .game
        .clone()
        .or_else(|| opts.discover_game.then(discover_game_path).flatten());
    if game.is_none() {
        eprintln!("analyze: no game path — pass --game <install>; scanning the workspace only");
    }
    let init = Request::build("initialize")
        .id(Id::Number(1))
        .params(json!({
            "processId": Value::Null,
            "capabilities": {},
            "workspaceFolders": folders,

            "initializationOptions": {
                "gamePath": game.as_ref().map(|g| g.to_string_lossy().to_string()),
                "workspaceScanEnabled": true,
                "scopeValidationEnabled": opts.scope_validation,
                "stylingEnabled": opts.styling,
                "logLevel": "info",
            },
        }))
        .finish();
    let init_result = match call(&mut service, init).await {
        Some(v) => v,
        None => {
            eprintln!("analyze: the server did not answer initialize");
            return 2;
        }
    };
    let legend: Vec<String> = init_result
        .pointer("/capabilities/semanticTokensProvider/legend/tokenTypes")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|t| t.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();

    let scan_start = Instant::now();
    notify(&mut service, "initialized", json!({})).await;
    let scanned = pump_until(&mut rx, opts.timeout, |req| {
        req.method() == "window/logMessage"
            && req
                .params()
                .and_then(|p| p.get("message"))
                .and_then(Value::as_str)
                .is_some_and(|m| m.contains("All scanners completed"))
    })
    .await
    .is_some();
    let scan_ms = scan_start.elapsed().as_millis();
    if !scanned {
        eprintln!("analyze: the workspace scan did not report completion; results may be partial");
    }

    notify(
        &mut service,
        "textDocument/didOpen",
        json!({"textDocument": {
            "uri": uri_str,
            "languageId": "hoi4",
            "version": 1,
            "text": content,
        }}),
    )
    .await;
    let diags_msg = pump_until(&mut rx, opts.timeout, |req| {
        req.method() == "textDocument/publishDiagnostics"
            && req
                .params()
                .and_then(|p| p.get("uri"))
                .and_then(Value::as_str)
                .is_some_and(|u| same_uri(u, &uri_str))
    })
    .await;
    let Some(diags_msg) = diags_msg else {
        eprintln!("analyze: no diagnostics were published for this file");
        return 2;
    };
    let diags: Vec<Value> = diags_msg
        .params()
        .and_then(|p| p.get("diagnostics"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut tokens: Vec<Value> = Vec::new();
    if opts.tokens {
        let req = Request::build("textDocument/semanticTokens/full")
            .id(Id::Number(2))
            .params(json!({"textDocument": {"uri": uri_str}}))
            .finish();
        let Some(resp) = call(&mut service, req).await else {
            eprintln!("analyze: no semantic tokens returned");
            return 2;
        };
        let data = resp
            .pointer("/data")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        tokens = decode_tokens(&data, &legend, &content);
    }

    if opts.json {
        let payload = json!({
            "path": path,
            "workspace": workspaces,
            "game": game,
            "scan_ms": scan_ms,
            "scanned": scanned,
            "diagnostics": diags.iter().map(|d| {
                let start = d.get("range").and_then(|r| r.get("start"));
                json!({
                    "code": d.get("code").cloned().unwrap_or(Value::Null),
                    "severity": d.get("severity").cloned().unwrap_or(Value::Null),
                    "message": d.get("message").cloned().unwrap_or(Value::Null),
                    "line": start.and_then(|s| s.get("line")).and_then(Value::as_u64).map(|l| l + 1),
                    "col": start.and_then(|s| s.get("character")).and_then(Value::as_u64).map(|c| c + 1),
                })
            }).collect::<Vec<_>>(),
            "tokens": tokens,
        });
        println!("{}", serde_json::to_string(&payload).unwrap_or_default());
    } else {
        println!("path:      {}", path.display());
        for ws in &workspaces {
            println!("workspace: {}", ws.display());
        }
        if let Some(g) = &game {
            println!("game:      {}", g.display());
        }
        println!(
            "scan:      {scan_ms} ms{}",
            if scanned { "" } else { " (incomplete)" }
        );
        println!("diagnostics: {}", diags.len());
        for d in &diags {
            let start = d.get("range").and_then(|r| r.get("start"));
            let line = start
                .and_then(|s| s.get("line"))
                .and_then(Value::as_u64)
                .map_or(0, |l| l + 1);
            let col = start
                .and_then(|s| s.get("character"))
                .and_then(Value::as_u64)
                .map_or(0, |c| c + 1);
            let code = match d.get("code") {
                Some(Value::String(s)) => s.clone(),
                Some(Value::Number(n)) => n.to_string(),
                _ => "?".to_owned(),
            };
            let sev = match d.get("severity").and_then(Value::as_u64) {
                Some(1) => "error",
                Some(2) => "warning",
                Some(3) => "info",
                Some(4) => "hint",
                _ => "?",
            };
            let msg = d.get("message").and_then(Value::as_str).unwrap_or("");
            println!("  {line}:{col}  {sev}  {code}  {msg}");
        }
        if opts.tokens {
            println!("tokens: {}", tokens.len());
            for t in &tokens {
                println!(
                    "  {}:{}  {:<28} {}",
                    t["line"],
                    t["col"],
                    t["text"].as_str().unwrap_or(""),
                    t["type"].as_str().unwrap_or("")
                );
            }
        }
    }

    if diags.is_empty() { 0 } else { 1 }
}

/// The workspace root is the nearest ancestor that holds a `common/` or
/// `events/` directory — the same rule the extension uses to spot a mod.
fn infer_workspace(file: &Path) -> PathBuf {
    let mut dir = file.parent().unwrap_or(file);
    loop {
        if dir.join("common").is_dir() || dir.join("events").is_dir() {
            return dir.to_path_buf();
        }
        match dir.parent() {
            Some(p) => dir = p,
            None => return file.parent().unwrap_or(file).to_path_buf(),
        }
    }
}

/// Steam's two install layouts. A wrong guess is cheap: the scan simply finds
/// no game files and the CLI says so.
fn discover_game_path() -> Option<PathBuf> {
    let home = std::env::var_os("HOME").map(PathBuf::from)?;
    [
        home.join(".local/share/Steam/steamapps/common/Hearts of Iron IV"),
        home.join(".steam/steam/steamapps/common/Hearts of Iron IV"),
    ]
    .into_iter()
    .find(|p| p.is_dir())
}

/// The folder list in the shape `initialize` and `workspace/workspaceFolders`
/// both expect.
fn folders_json(workspaces: &[PathBuf]) -> Value {
    Value::Array(
        workspaces
            .iter()
            .map(|w| {
                json!({
                    "uri": Uri::from_file_path(w).map(|u| u.to_string()),
                    "name": w
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default(),
                })
            })
            .collect(),
    )
}

fn same_uri(a: &str, b: &str) -> bool {
    // Drive letters and the escaping of spaces are the two places a client and
    // the server can spell the same file differently.
    a.eq_ignore_ascii_case(b) || a.replace("%20", " ") == b.replace("%20", " ")
}

/// Send one request and hand back its `result` — the envelope's id and version
/// are ours, not the caller's.
async fn call(service: &mut LspService<Backend>, req: Request) -> Option<Value> {
    let resp = service.call(req).await.ok().flatten()?;
    let mut value = serde_json::to_value(resp).ok()?;
    value.get_mut("result").map(Value::take)
}

async fn notify(service: &mut LspService<Backend>, method: &str, params: Value) {
    let _ = service
        .call(Request::build(method.to_owned()).params(params).finish())
        .await;
}

/// Read server-to-client messages until `pred` matches, or the timeout expires.
async fn pump_until<F>(
    rx: &mut tokio::sync::mpsc::UnboundedReceiver<Request>,
    timeout: Duration,
    mut pred: F,
) -> Option<Request>
where
    F: FnMut(&Request) -> bool,
{
    let deadline = Instant::now() + timeout;
    loop {
        let left = deadline.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return None;
        }
        match tokio::time::timeout(left, rx.recv()).await {
            Ok(Some(req)) => {
                if pred(&req) {
                    return Some(req);
                }
            }
            Ok(None) | Err(_) => return None,
        }
    }
}

/// Turn the delta-encoded `semanticTokens/full` payload into absolute ranges,
/// keeping the token text so a report reads without a second lookup.
fn decode_tokens(data: &[Value], legend: &[String], content: &str) -> Vec<Value> {
    let lines: Vec<&str> = content.split('\n').collect();
    let mut out = Vec::with_capacity(data.len() / 5);
    let (mut line, mut col) = (0i64, 0i64);
    for chunk in data.chunks(5) {
        let [delta_line, delta_start, length, kind, _mods] = chunk else {
            break;
        };
        let (Some(dl), Some(ds), Some(len), Some(kind)) = (
            delta_line.as_i64(),
            delta_start.as_i64(),
            length.as_u64(),
            kind.as_u64(),
        ) else {
            continue;
        };
        if dl != 0 {
            line += dl;
            col = ds;
        } else {
            col += ds;
        }
        let text = lines
            .get(line as usize)
            .map(|src| {
                let chars: Vec<char> = src.chars().collect();
                chars
                    .get(col as usize..(col as usize + len as usize).min(chars.len()))
                    .map(|s| s.iter().collect::<String>())
                    .unwrap_or_default()
            })
            .unwrap_or_default();
        out.push(json!({
            "line": line + 1,
            "col": col + 1,
            "length": len,
            "type": legend.get(kind as usize).cloned().unwrap_or_else(|| format!("?{kind}")),
            "text": text,
        }));
    }
    out
}
