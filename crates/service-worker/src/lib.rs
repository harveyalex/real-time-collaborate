use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{future_to_promise, JsFuture};

const CACHE_NAME: &str = "collaborate-v1";
const SHELL_ASSETS: &[&str] = &["/", "/index.html"];

fn global_scope() -> web_sys::ServiceWorkerGlobalScope {
    let global = js_sys::global();
    global.unchecked_into::<web_sys::ServiceWorkerGlobalScope>()
}

#[wasm_bindgen(start)]
pub fn start() {
    // Entry point — event listeners are registered here via JS glue.
}

/// Called by the JS glue on the `install` event.
#[wasm_bindgen]
pub fn on_install(event: web_sys::ExtendableEvent) {
    let promise = future_to_promise(async move {
        let scope = global_scope();
        let caches: web_sys::CacheStorage = scope.caches().map_err(|e| e)?;

        let cache: web_sys::Cache =
            JsFuture::from(caches.open(CACHE_NAME)).await?.unchecked_into();

        let urls = js_sys::Array::new();
        for &asset in SHELL_ASSETS {
            urls.push(&JsValue::from_str(asset));
        }

        JsFuture::from(cache.add_all_with_str_sequence(&urls)).await?;

        Ok(JsValue::UNDEFINED)
    });

    event.wait_until(&promise).unwrap_or_default();
}

/// Called by the JS glue on the `fetch` event.
#[wasm_bindgen]
pub fn on_fetch(event: web_sys::FetchEvent) {
    let request: web_sys::Request = event.request();
    let url = request.url();

    // Skip WebSocket and /v1/ API paths — let them go straight to the network.
    if url.contains("ws://")
        || url.contains("wss://")
        || url.contains("/v1/")
    {
        return;
    }

    let promise = future_to_promise(async move {
        let scope = global_scope();
        let caches: web_sys::CacheStorage = scope.caches().map_err(|e| e)?;

        // Try cache first.
        let cached = JsFuture::from(caches.match_with_str(&url)).await?;
        if !cached.is_undefined() && !cached.is_null() {
            return Ok(cached);
        }

        // Cache miss — fetch from network.
        let network_response: web_sys::Response =
            JsFuture::from(scope.fetch_with_str(&url))
                .await?
                .unchecked_into();

        // Only cache valid responses (status 200, basic/cors type).
        if network_response.ok() {
            // Clone before consuming — the body can only be read once.
            let response_to_cache = network_response.clone()?;
            let cache: web_sys::Cache =
                JsFuture::from(caches.open(CACHE_NAME)).await?.unchecked_into();
            JsFuture::from(cache.put_with_str(&url, &response_to_cache)).await?;
        }

        Ok(network_response.into())
    });

    event.respond_with(&promise).unwrap_or_default();
}
