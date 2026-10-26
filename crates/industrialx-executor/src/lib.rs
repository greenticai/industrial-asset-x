//! Deterministic, advisory-only native executor for Industrial-X playbooks.
//!
//! `handle` is the whole service as a pure function over (method, path, body);
//! `http` puts it on a loopback socket. No network egress, no credentials, no
//! control path: see `guard`.

mod execute;
mod features;
pub mod guard;
pub mod http;
mod rules;
mod scoring;
pub mod validate;

use serde_json::{Value, json};

use execute::Refusal;

/// Largest request body accepted, in bytes.
pub const MAX_BODY_BYTES: usize = 256 * 1024;

#[derive(Debug, Clone, PartialEq)]
pub struct Response {
    pub status: u16,
    pub body: Value,
}

impl Response {
    fn ok(body: Value) -> Self {
        Self { status: 200, body }
    }

    fn refusal(refusal: &Refusal) -> Self {
        Self {
            status: refusal.status,
            body: refusal.to_json(),
        }
    }
}

fn parse(body: &[u8]) -> Result<Value, Refusal> {
    if body.len() > MAX_BODY_BYTES {
        return Err(Refusal::new(
            413,
            "body_too_large",
            format!("the body exceeds {MAX_BODY_BYTES} bytes"),
        ));
    }
    serde_json::from_slice(body).map_err(|error| {
        Refusal::new(
            400,
            "invalid_json",
            format!("the body is not JSON: {error}"),
        )
    })
}

fn refuse_unsafe(request: &Value, control: bool) -> Result<(), Refusal> {
    if control && let Some(message) = guard::control_request(request) {
        return Err(Refusal::new(422, "control_refused", message));
    }
    if let Some(message) = guard::secret_in_request(request) {
        return Err(Refusal::new(422, "secret_refused", message));
    }
    Ok(())
}

fn validate_route(body: &[u8]) -> Result<Value, Refusal> {
    let request = match parse(body) {
        Ok(request) => request,
        // /validate answers 200 for invalid input; only an oversized body is a transport error.
        Err(refusal) if refusal.status != 413 => {
            return Ok(json!({
                "valid": false,
                "errors": [{"code": refusal.code, "field": "", "message": refusal.message}],
                "playbook_id": null,
            }));
        }
        Err(refusal) => return Err(refusal),
    };
    refuse_unsafe(&request, false)?;
    let playbook = request.get("playbook").unwrap_or(&request);
    let errors = validate::errors(playbook);
    Ok(json!({
        "valid": errors.is_empty(),
        "errors": errors,
        "playbook_id": playbook.get("id").and_then(Value::as_str),
        "warnings": if errors.is_empty() { validate::findings(playbook) } else { Vec::new() },
    }))
}

fn execute_route(body: &[u8]) -> Result<Value, Refusal> {
    let request = parse(body)?;
    refuse_unsafe(&request, true)?;
    execute::execute(&request)
}

/// Handles one request. The body is never logged or echoed.
pub fn handle(method: &str, path: &str, body: &[u8]) -> Response {
    let path = path.split('?').next().unwrap_or(path);
    let result = match (method, path) {
        ("GET", "/healthz") => Ok(json!({"ok": true, "mode": "advisory"})),
        ("POST", "/validate") => validate_route(body),
        ("POST", "/execute") => execute_route(body),
        (_, "/healthz" | "/validate" | "/execute") => Err(Refusal::new(
            405,
            "method_not_allowed",
            format!("{method} is not allowed on {path}"),
        )),
        _ => Err(Refusal::new(404, "not_found", format!("no route {path}"))),
    };
    match result {
        Ok(body) => Response::ok(body),
        Err(refusal) => Response::refusal(&refusal),
    }
}
