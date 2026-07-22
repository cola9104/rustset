use serde_json::Value;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{ReadableStreamDefaultReader, Request, RequestInit, RequestMode};

/// Post to SSE endpoint and call on_chunk for each data event, on_done when complete
pub async fn sse_post(url: &str, body: &Value, token: &str, mut on_chunk: impl FnMut(String)) {
    let mut opts = RequestInit::new();
    opts.method("POST").mode(RequestMode::Cors);
    let headers = web_sys::Headers::new().unwrap();
    let _ = headers.set("Content-Type", "application/json");
    let _ = headers.set("Authorization", &format!("Bearer {}", token));
    let _ = headers.set("Accept", "text/event-stream");
    opts.headers(&headers);
    opts.body(Some(&wasm_bindgen::JsValue::from_str(&body.to_string())));

    let request = match Request::new_with_str_and_init(url, &opts) {
        Ok(r) => r,
        Err(_) => return,
    };
    let window = match web_sys::window() {
        Some(w) => w,
        None => return,
    };
    let resp = match JsFuture::from(window.fetch_with_request(&request)).await {
        Ok(r) => r,
        Err(_) => return,
    };
    let resp: web_sys::Response = match resp.dyn_into() {
        Ok(r) => r,
        Err(_) => return,
    };
    if !resp.ok() { return; }
    let body = match resp.body() {
        Some(b) => b,
        None => return,
    };
    let reader: ReadableStreamDefaultReader = match body.get_reader().dyn_into() {
        Ok(r) => r,
        Err(_) => return,
    };

    let mut buffer = String::new();
    loop {
        let chunk = match JsFuture::from(reader.read()).await {
            Ok(c) => c,
            Err(_) => break,
        };
        let done = js_sys::Reflect::get(&chunk, &"done".into()).ok()
            .and_then(|v| v.as_bool()).unwrap_or(false);
        if done { break; }
        let value = js_sys::Reflect::get(&chunk, &"value".into()).ok();
        if let Some(val) = value {
            if let Some(arr) = val.dyn_ref::<js_sys::Uint8Array>() {
                let bytes = arr.to_vec();
                if let Ok(s) = String::from_utf8(bytes) {
                    buffer.push_str(&s);
                    while let Some(pos) = buffer.find('\n') {
                        let line = buffer[..pos].trim().to_string();
                        buffer = buffer[pos + 1..].to_string();
                        if buffer.starts_with("data: ") {
                            buffer = buffer[6..].to_string();
                            continue;
                        }
                        if line.starts_with("data: ") {
                            let data = line[6..].trim().to_string();
                            if data == "[DONE]" { return; }
                            on_chunk(data);
                        }
                    }
                }
            }
        }
    }
}
