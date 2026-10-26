//! Minimal blocking HTTP/1.1 front for `handle`, bound to 127.0.0.1 only.
//! Bounded header and body size, a per-request deadline, a connection cap,
//! and nothing from the request body is ever logged.

use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::json;

use crate::{MAX_BODY_BYTES, Response, handle};

const MAX_HEADER_BYTES: usize = 16 * 1024;
const REQUEST_DEADLINE: Duration = Duration::from_secs(10);
const MAX_CONNECTIONS: usize = 32;

/// Binds loopback only; there is deliberately no address option.
pub fn bind(port: u16) -> std::io::Result<TcpListener> {
    TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, port)))
}

pub fn serve(port: u16) -> std::io::Result<()> {
    let listener = bind(port)?;
    println!(
        "industrialx-executor listening on {}",
        listener.local_addr()?
    );
    serve_on(listener);
    Ok(())
}

pub fn serve_on(listener: TcpListener) {
    let active = Arc::new(AtomicUsize::new(0));
    for stream in listener.incoming().flatten() {
        if active.fetch_add(1, Ordering::SeqCst) >= MAX_CONNECTIONS {
            active.fetch_sub(1, Ordering::SeqCst);
            let _ = write_response(&stream, &error(503, "busy", "too many connections"));
            continue;
        }
        let active = Arc::clone(&active);
        thread::spawn(move || {
            let response = serve_connection(&stream);
            let _ = write_response(&stream, &response);
            active.fetch_sub(1, Ordering::SeqCst);
        });
    }
}

fn error(status: u16, code: &str, message: &str) -> Response {
    Response {
        status,
        body: json!({"error": {"code": code, "message": message}}),
    }
}

struct Head {
    method: String,
    path: String,
    content_length: usize,
}

fn parse_head(head: &str) -> Result<Head, Response> {
    let mut lines = head.split("\r\n");
    let request_line = lines.next().unwrap_or_default();
    let mut parts = request_line.split(' ');
    let (Some(method), Some(path)) = (parts.next(), parts.next()) else {
        return Err(error(400, "bad_request", "malformed request line"));
    };
    let mut content_length = 0usize;
    for line in lines {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        let name = name.trim();
        if name.eq_ignore_ascii_case("transfer-encoding") {
            return Err(error(
                400,
                "bad_request",
                "chunked bodies are not supported",
            ));
        }
        if name.eq_ignore_ascii_case("content-length") {
            content_length = value
                .trim()
                .parse()
                .map_err(|_| error(400, "bad_request", "bad Content-Length"))?;
        }
    }
    Ok(Head {
        method: method.to_owned(),
        path: path.to_owned(),
        content_length,
    })
}

fn serve_connection(mut stream: &TcpStream) -> Response {
    let started = Instant::now();
    let mut buffer: Vec<u8> = Vec::new();
    let mut chunk = [0u8; 4096];
    let header_end = loop {
        if let Some(at) = buffer.windows(4).position(|w| w == b"\r\n\r\n") {
            break at;
        }
        if buffer.len() > MAX_HEADER_BYTES {
            return error(431, "headers_too_large", "request headers are too large");
        }
        match read_some(&mut stream, &mut chunk, started) {
            Ok(0) => return error(400, "bad_request", "connection closed before headers ended"),
            Ok(n) => buffer.extend_from_slice(&chunk[..n]),
            Err(response) => return response,
        }
    };
    let head = match std::str::from_utf8(&buffer[..header_end])
        .map_err(|_| error(400, "bad_request", "headers are not UTF-8"))
        .and_then(parse_head)
    {
        Ok(head) => head,
        Err(response) => return response,
    };
    // Decide on the declared size before reading any of the body.
    if head.content_length > MAX_BODY_BYTES {
        return error(
            413,
            "body_too_large",
            &format!("the body exceeds {MAX_BODY_BYTES} bytes"),
        );
    }
    let mut body = buffer[header_end + 4..].to_vec();
    while body.len() < head.content_length {
        match read_some(&mut stream, &mut chunk, started) {
            Ok(0) => {
                return error(
                    400,
                    "bad_request",
                    "connection closed before the body ended",
                );
            }
            Ok(n) => body.extend_from_slice(&chunk[..n]),
            Err(response) => return response,
        }
    }
    body.truncate(head.content_length);
    handle(&head.method, &head.path, &body)
}

fn read_some(
    stream: &mut &TcpStream,
    chunk: &mut [u8],
    started: Instant,
) -> Result<usize, Response> {
    let left = REQUEST_DEADLINE.saturating_sub(started.elapsed());
    if left.is_zero() {
        return Err(error(408, "timeout", "the request took too long"));
    }
    stream
        .set_read_timeout(Some(left))
        .map_err(|_| error(500, "internal", "could not set a timeout"))?;
    stream.read(chunk).map_err(|e| match e.kind() {
        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut => {
            error(408, "timeout", "the request took too long")
        }
        _ => error(400, "bad_request", "could not read the request"),
    })
}

fn write_response(mut stream: &TcpStream, response: &Response) -> std::io::Result<()> {
    let body = serde_json::to_vec(&response.body).unwrap_or_else(|_| b"{}".to_vec());
    let head = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        response.status,
        reason(response.status),
        body.len()
    );
    stream.set_write_timeout(Some(REQUEST_DEADLINE))?;
    stream.write_all(head.as_bytes())?;
    stream.write_all(&body)
}

fn reason(status: u16) -> &'static str {
    match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        408 => "Request Timeout",
        413 => "Payload Too Large",
        422 => "Unprocessable Entity",
        431 => "Request Header Fields Too Large",
        503 => "Service Unavailable",
        _ => "Internal Server Error",
    }
}
