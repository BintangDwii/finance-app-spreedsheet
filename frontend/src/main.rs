//! Corporate Finance — Leptos CSR frontend (WASM SPA).
//! Same-origin `/api/*` calls; session cookie (`jwt`, HttpOnly) is sent
//! automatically by the browser. No token storage in JS (§7).

mod api;
mod app;
mod components;
mod pages;

use app::App;
use leptos::prelude::*;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}
