//! Cloudflare Worker entry (wasm32 only). Same routes, CORS and JSON as the old src/index.js.
use crate::core::handle;
use crate::page::page;
use worker::*;

fn cors(r: Response) -> Result<Response> {
    let mut r = r;
    r.headers_mut().set("content-type", "application/json")?;
    r.headers_mut().set("access-control-allow-origin", "*")?;
    Ok(r)
}

#[event(fetch)]
async fn fetch(mut req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    let path = req.url()?.path().to_string();
    let method = req.method();
    let body = if method == Method::Post { req.text().await.unwrap_or_default() } else { String::new() };
    let (status, v) = handle(method.as_ref(), &path, &body, &|| js_sys::Date::now() as i64);
    if status == 0 {
        let h = Headers::new();
        h.set("content-type", "text/html; charset=utf-8")?;
        return Ok(Response::ok(page())?.with_headers(h));
    }
    cors(Response::from_json(&v)?.with_status(status))
}
