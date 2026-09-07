//! Opt-in localhost bridge for browser tests. Not included in desktop releases.
use gen3_core::app::{App, Request};
use std::io::Read;
fn main() {
    let token = std::env::var("GEN3_DEV_TOKEN")
        .expect("set GEN3_DEV_TOKEN to a random 32+ character string");
    assert!(token.len() >= 32);
    let server = tiny_http::Server::http("127.0.0.1:8766").expect("bind development bridge");
    let mut app = App::default();
    eprintln!("Development bridge: 127.0.0.1:8766");
    for mut request in server.incoming_requests() {
        let authorized = request
            .headers()
            .iter()
            .any(|h| h.field.equiv("X-Gen3-Token") && h.value.as_str() == token);
        if !authorized || request.method() != &tiny_http::Method::Post || request.url() != "/api" {
            let _ = request
                .respond(tiny_http::Response::from_string("Forbidden").with_status_code(403));
            continue;
        }
        let mut body = Vec::new();
        if request
            .as_reader()
            .take(50_000_001)
            .read_to_end(&mut body)
            .is_err()
            || body.len() > 50_000_000
        {
            let _ = request
                .respond(tiny_http::Response::from_string("Too large").with_status_code(413));
            continue;
        }
        let result = serde_json::from_slice::<Request>(&body)
            .map_err(gen3_core::Error::from)
            .and_then(|r| app.dispatch(r));
        let json = match result {
            Ok(data) => serde_json::json!({"ok":true,"data":data}),
            Err(error) => serde_json::json!({"ok":false,"error":error}),
        };
        let mut response = tiny_http::Response::from_string(json.to_string());
        response
            .add_header(tiny_http::Header::from_bytes("Content-Type", "application/json").unwrap());
        let _ = request.respond(response);
    }
}
