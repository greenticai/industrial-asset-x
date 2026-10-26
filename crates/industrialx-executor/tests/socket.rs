use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;

fn start() -> std::net::SocketAddr {
    let listener = industrialx_executor::http::bind(0).expect("bind loopback");
    let addr = listener.local_addr().expect("addr");
    thread::spawn(move || industrialx_executor::http::serve_on(listener));
    addr
}

fn exchange(addr: std::net::SocketAddr, raw: &[u8]) -> String {
    let mut stream = TcpStream::connect(addr).expect("connect");
    stream.write_all(raw).expect("write");
    let mut out = String::new();
    stream.read_to_string(&mut out).expect("read");
    out
}

#[test]
fn binds_loopback_only_and_serves_validate() {
    let addr = start();
    assert!(addr.ip().is_loopback());
    let body = r#"{"playbook": 5}"#;
    let raw = format!(
        "POST /validate HTTP/1.1\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );
    let out = exchange(addr, raw.as_bytes());
    assert!(out.starts_with("HTTP/1.1 200"), "{out}");
    assert!(out.contains("\"valid\":false"), "{out}");
}

#[test]
fn an_oversized_declared_body_is_refused_before_it_is_read() {
    let addr = start();
    let raw = "POST /execute HTTP/1.1\r\nContent-Length: 10000000\r\n\r\n";
    let out = exchange(addr, raw.as_bytes());
    assert!(out.starts_with("HTTP/1.1 413"), "{out}");
}

#[test]
fn chunked_bodies_are_refused() {
    let addr = start();
    let raw = "POST /execute HTTP/1.1\r\nTransfer-Encoding: chunked\r\n\r\n0\r\n\r\n";
    assert!(exchange(addr, raw.as_bytes()).starts_with("HTTP/1.1 400"));
}
