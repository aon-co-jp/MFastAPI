//! MFastAPI HTTP server layer (Rust). Routing uses the Starlette-compatible route table from `mfastapi-core`; validation errors are
//! FastAPI/Pydantic-identical 422 responses; `/openapi.json` (OpenAPI 3.1) and `/docs` are served. Handlers are plain Rust closures
//! (stage C) or call into embedded Python (stage B, feature `python`). GET/HEAD/POST/PUT/PATCH/DELETE with JSON bodies; a path match with the wrong method gives 405 + Allow.
use mfastapi_core::matcher::Table;
use mfastapi_core::parse_int;
use crate::cpu::query_last;
use serde_json::{json, Map, Value};
use std::collections::HashMap;

pub mod cpu;
pub mod update;
#[cfg(feature = "python")]
pub mod pybridge;
#[cfg(feature = "rmojo")]
pub mod rmojobridge;
pub mod server;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Method {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}
impl Method {
    pub fn parse(s: &str) -> Option<Method> {
        Some(match s {
            "GET" | "HEAD" => Method::Get,
            "POST" => Method::Post,
            "PUT" => Method::Put,
            "PATCH" => Method::Patch,
            "DELETE" => Method::Delete,
            _ => return None,
        })
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            Method::Get => "GET",
            Method::Post => "POST",
            Method::Put => "PUT",
            Method::Patch => "PATCH",
            Method::Delete => "DELETE",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Int,
    Str,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Loc {
    Path,
    Query,
}

pub struct Param {
    pub name: &'static str,
    pub loc: Loc,
    pub kind: Kind,
    pub required: bool,
    pub default_int: i64,
}

impl Param {
    pub fn path(name: &'static str, kind: Kind) -> Param {
        Param { name, loc: Loc::Path, kind, required: true, default_int: 0 }
    }
    pub fn query(name: &'static str, kind: Kind) -> Param {
        Param { name, loc: Loc::Query, kind, required: true, default_int: 0 }
    }
    pub fn default_int(mut self, v: i64) -> Param {
        self.required = false;
        self.default_int = v;
        self
    }
}

pub enum Val {
    Int(i64),
    Str(String),
}

#[derive(Default)]
pub struct Args(HashMap<&'static str, Val>, Option<Value>);

impl Args {
    pub fn int(&self, name: &str) -> i64 {
        match self.0.get(name) {
            Some(Val::Int(v)) => *v,
            _ => 0,
        }
    }
    /// Parsed JSON request body (None when the route takes no body).
    pub fn json(&self) -> Option<&Value> {
        self.1.as_ref()
    }
    pub fn str(&self, name: &str) -> &str {
        match self.0.get(name) {
            Some(Val::Str(s)) => s,
            _ => "",
        }
    }
}

pub struct Resp {
    pub status: u16,
    pub ctype: &'static str,
    pub body: String,
    pub allow: Option<String>,
}

impl Resp {
    pub fn json(status: u16, body: String) -> Resp {
        Resp { status, ctype: "application/json", body, allow: None }
    }
    pub fn detail(status: u16, msg: &str) -> Resp {
        Resp::json(status, json!({ "detail": msg }).to_string())
    }
}

pub type Handler = Box<dyn Fn(&Args) -> Resp + Send + Sync>;

pub struct Route {
    pub method: Method,
    /// Route reads a required JSON body (FastAPI body parameter).
    pub body: bool,
    pub path: &'static str, // FastAPI style, e.g. "/fib/{n}"
    pub name: &'static str,
    pub params: Vec<Param>,
    pub returns: Kind,
    pub handler: Handler,
}

pub struct App {
    routes: Vec<Route>,
    table: Table,
}

fn hexval(c: u8) -> Option<u8> {
    (c as char).to_digit(16).map(|d| d as u8)
}

/// Percent-decode a URL path like Starlette (`unquote`: bytes -> UTF-8 with replacement; `+` stays `+`).
fn unquote(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let (Some(h), Some(l)) = (hexval(b[i + 1]), hexval(b[i + 2])) {
                out.push(h * 16 + l);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn title(name: &str) -> String {
    let mut out = String::new();
    let mut up = true;
    for c in name.chars() {
        if c == '_' {
            out.push(' ');
            up = true;
        } else if up {
            out.extend(c.to_uppercase());
            up = false;
        } else {
            out.push(c);
        }
    }
    out
}

fn op_id(name: &str, path: &str, method: Method) -> String {
    let mut s: String = name.chars().chain(path.chars()).map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect();
    s.push('_');
    s.push_str(&method.as_str().to_lowercase());
    s
}

impl App {
    pub fn new(routes: Vec<Route>) -> Result<App, String> {
        let mut table = Table::new();
        for r in &routes {
            table.add(r.path.as_bytes()).map_err(|c| format!("bad route {}: {}", r.path, c))?;
        }
        Ok(App { routes, table })
    }

    pub fn openapi(&self) -> String {
        let mut paths = Map::new();
        for r in &self.routes {
            let params: Vec<Value> = r
                .params
                .iter()
                .map(|p| {
                    let mut schema = Map::new();
                    schema.insert("type".into(), json!(if p.kind == Kind::Int { "integer" } else { "string" }));
                    if !p.required && p.kind == Kind::Int {
                        schema.insert("default".into(), json!(p.default_int));
                    }
                    schema.insert("title".into(), json!(title(p.name)));
                    json!({"name": p.name, "in": if p.loc == Loc::Path { "path" } else { "query" }, "required": p.required, "schema": schema})
                })
                .collect();
            let mut op = Map::new();
            op.insert("summary".into(), json!(title(r.name)));
            op.insert("operationId".into(), json!(op_id(r.name, r.path, r.method)));
            if !params.is_empty() {
                op.insert("parameters".into(), json!(params));
            }
            if r.body {
                op.insert("requestBody".into(), json!({"required": true, "content": {"application/json": {"schema": {}}}}));
            }
            let mut responses = Map::new();
            responses.insert(
                "200".into(),
                json!({"description": "Successful Response", "content": {"application/json": {"schema": {
                    "type": if r.returns == Kind::Int { "integer" } else { "string" }, "title": format!("Response {}", title(r.name))}}}}),
            );
            if !r.params.is_empty() || r.body {
                responses.insert(
                    "422".into(),
                    json!({"description": "Validation Error", "content": {"application/json": {"schema": {"$ref": "#/components/schemas/HTTPValidationError"}}}}),
                );
            }
            op.insert("responses".into(), Value::Object(responses));
            let entry = paths.entry(r.path.to_string()).or_insert_with(|| json!({}));
            entry[r.method.as_str().to_lowercase()] = Value::Object(op);
        }
        json!({
            "openapi": "3.1.0",
            "info": {"title": "FastAPI", "version": "0.1.0"},
            "paths": paths,
            "components": {"schemas": {
                "HTTPValidationError": {"properties": {"detail": {"items": {"$ref": "#/components/schemas/ValidationError"}, "type": "array", "title": "Detail"}}, "type": "object", "title": "HTTPValidationError"},
                "ValidationError": {"properties": {"loc": {"items": {"anyOf": [{"type": "string"}, {"type": "integer"}]}, "type": "array", "title": "Location"},
                    "msg": {"type": "string", "title": "Message"}, "type": {"type": "string", "title": "Error Type"}, "input": {"title": "Input"}, "ctx": {"type": "object", "title": "Context"}},
                    "type": "object", "required": ["loc", "msg", "type"], "title": "ValidationError"}
            }}
        })
        .to_string()
    }

    pub fn docs_html() -> String {
        concat!(
            "<!DOCTYPE html><html><head><link type=\"text/css\" rel=\"stylesheet\" href=\"https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui.css\">",
            "<title>FastAPI - Swagger UI</title></head><body><div id=\"swagger-ui\"></div>",
            "<script src=\"https://cdn.jsdelivr.net/npm/swagger-ui-dist@5/swagger-ui-bundle.js\"></script>",
            "<script>const ui = SwaggerUIBundle({url: \"/openapi.json\", dom_id: \"#swagger-ui\", presets: [SwaggerUIBundle.presets.apis, SwaggerUIBundle.SwaggerUIStandalonePreset], layout: \"BaseLayout\", deepLinking: true})</script></body></html>"
        )
        .to_string()
    }

    /// Handle one request (raw, still percent-encoded path, raw query string, raw body bytes).
    pub fn handle(&self, method: &str, raw_path: &str, query: &str, body: &[u8]) -> Resp {
        let path = unquote(raw_path);
        let m = match Method::parse(method) {
            Some(m) => m,
            None => return Resp::detail(405, "Method Not Allowed"),
        };
        if m == Method::Get {
            match path.as_str() {
                "/openapi.json" => return Resp::json(200, self.openapi()),
                "/docs" => return Resp { status: 200, ctype: "text/html; charset=utf-8", body: App::docs_html(), allow: None },
                "/health" => return Resp::json(200, "{\"status\":\"ok\"}".into()),
                _ => {}
            }
        }
        let mut start = 0usize;
        let mut allowed: Vec<&'static str> = Vec::new();
        let mut found: Option<usize> = None;
        loop {
            let idx = self.table.first_match_from(path.as_bytes(), start as i32);
            if idx < 0 {
                break;
            }
            let i = idx as usize;
            if self.routes[i].method == m {
                found = Some(i);
                break;
            }
            let name = self.routes[i].method.as_str();
            if !allowed.contains(&name) {
                allowed.push(name);
            }
            start = i + 1;
        }
        let idx = match found {
            Some(i) => i,
            None if !allowed.is_empty() => {
                let mut r = Resp::detail(405, "Method Not Allowed");
                r.allow = Some(allowed.join(", "));
                return r;
            }
            None => return Resp::detail(404, "Not Found"),
        };
        let route = &self.routes[idx];
        // path params: whole-segment {name} / {name:type}
        let mut pvals: HashMap<&str, &str> = HashMap::new();
        let tsegs: Vec<&str> = route.path.split('/').collect();
        let psegs: Vec<&str> = path.split('/').collect();
        if !route.path.contains(":path}") && tsegs.len() == psegs.len() {
            for (t, p) in tsegs.iter().zip(psegs.iter()) {
                if t.starts_with('{') && t.ends_with('}') {
                    let name = t[1..t.len() - 1].split(':').next().unwrap_or("");
                    pvals.insert(name, p);
                }
            }
        }
        let mut errors: Vec<Value> = Vec::new();
        let mut args = Args::default();
        let mut ordered: Vec<&Param> = route.params.iter().filter(|p| p.loc == Loc::Path).collect();
        ordered.extend(route.params.iter().filter(|p| p.loc == Loc::Query));
        for p in ordered {
            let loc = if p.loc == Loc::Path { "path" } else { "query" };
            let raw: Option<String> = match p.loc {
                Loc::Path => pvals.get(p.name).map(|s| s.to_string()),
                Loc::Query => query_last(query.as_bytes(), p.name.as_bytes()).map(|v| String::from_utf8_lossy(&v).into_owned()),
            };
            match raw {
                None if p.required => {
                    errors.push(json!({"type": "missing", "loc": [loc, p.name], "msg": "Field required", "input": Value::Null}));
                }
                None => {
                    args.0.insert(p.name, Val::Int(p.default_int));
                }
                Some(s) => match p.kind {
                    Kind::Str => {
                        args.0.insert(p.name, Val::Str(s));
                    }
                    Kind::Int => match parse_int(s.as_bytes()) {
                        Ok(v) => {
                            args.0.insert(p.name, Val::Int(v));
                        }
                        Err(_) => {
                            errors.push(json!({"type": "int_parsing", "loc": [loc, p.name],
                                "msg": "Input should be a valid integer, unable to parse string as an integer", "input": s}));
                        }
                    },
                },
            }
        }
        if route.body {
            if body.iter().all(|c| c.is_ascii_whitespace()) {
                errors.push(json!({"type": "missing", "loc": ["body"], "msg": "Field required", "input": Value::Null}));
            } else {
                match serde_json::from_slice::<Value>(body) {
                    Ok(v) => args.1 = Some(v),
                    Err(e) => errors.push(json!({"type": "json_invalid", "loc": ["body", e.column().saturating_sub(1)],
                        "msg": "JSON decode error", "input": {}, "ctx": {"error": e.to_string()}})),
                }
            }
        }
        if !errors.is_empty() {
            return Resp::json(422, json!({ "detail": errors }).to_string());
        }
        (route.handler)(&args)
    }
}
