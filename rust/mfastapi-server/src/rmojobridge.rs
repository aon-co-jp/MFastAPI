//! RMojo bridge: handlers written in Mojo-style source run in the memory-safe RMojo interpreter (no Python, no GIL).
//! One dedicated worker thread (large stack, program compiled once) executes calls; each call has a loop-step budget.
use crate::{Args, Handler, Kind, Resp};
use rmojo::{CompiledProgram, Limits, Value};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};

type Job = (Vec<Value>, Sender<Result<Value, String>>);

/// Build a handler that calls `func` of `source` with `params` (name, kind) taken from the request.
/// Returns int/str as JSON. Errors: step limit -> 504, other handler errors -> 400 (detail), worker gone -> 503.
pub fn rmojo_handler(source: &str, func: &str, params: Vec<(&'static str, Kind)>, returns: Kind, max_steps: u64) -> Handler {
    let (tx, rx) = channel::<Job>();
    let (src, f) = (source.to_string(), func.to_string());
    std::thread::Builder::new()
        .stack_size(1 << 28)
        .spawn(move || {
            let prog: Result<CompiledProgram, String> = rmojo::compile_source(&src);
            let lim = Limits { max_steps, max_ms: None };
            while let Ok((args, reply)) = rx.recv() {
                let r = match &prog {
                    Ok(p) => p.call_limited(&f, &args, &lim),
                    Err(e) => Err(e.clone()),
                };
                let _ = reply.send(r);
            }
        })
        .expect("spawn rmojo worker");
    let tx = Arc::new(Mutex::new(tx));
    Box::new(move |a: &Args| {
        let vals: Vec<Value> = params
            .iter()
            .map(|(n, k)| match k {
                Kind::Int => Value::Int(a.int(n)),
                Kind::Str => Value::Str(a.str(n).to_string()),
            })
            .collect();
        let (rtx, rrx) = channel();
        if tx.lock().map(|t| t.send((vals, rtx)).is_err()).unwrap_or(true) {
            return Resp::detail(503, "rmojo worker unavailable");
        }
        match rrx.recv() {
            Ok(Ok(Value::Int(i))) if returns == Kind::Int => Resp::json(200, i.to_string()),
            Ok(Ok(Value::Str(s))) if returns == Kind::Str => Resp::json(200, serde_json::Value::String(s).to_string()),
            Ok(Ok(_)) => Resp::detail(500, "rmojo handler returned an unexpected type"),
            Ok(Err(e)) if e.starts_with("LimitError") => Resp::detail(504, "handler exceeded its step limit"),
            Ok(Err(e)) => Resp::detail(400, &e),
            Err(_) => Resp::detail(503, "rmojo worker unavailable"),
        }
    })
}
