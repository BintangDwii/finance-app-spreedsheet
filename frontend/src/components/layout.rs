//! App shell: sidebar nav + outlet. Auth gate lives in `app.rs`.

use leptos::prelude::*;
use leptos_router::{components::A, hooks::use_navigate};

use crate::api::UserPublic;

#[component]
pub fn Layout(user: UserPublic, children: Children) -> impl IntoView {
    let uname = user.username.clone();
    let role = format!("{:?}", user.role);
    view! {
        <div class="min-h-screen bg-slate-100 text-slate-900 flex">
            <aside class="w-60 shrink-0 bg-slate-900 text-slate-100 p-4 flex flex-col gap-1">
                <div class="text-lg font-bold px-2 py-3">"🏢 Corporate Finance"</div>
                <div class="text-xs text-slate-400 px-2 pb-3">{format!("{uname} • {role}")}</div>
                <NavLink href="/" label="📊 Dashboard" />
                <NavLink href="/input" label="➕ Input" />
                <NavLink href="/approval" label="✅ Approval" />
                <NavLink href="/accounts" label="🏦 Rekening" />
                <NavLink href="/budgets" label="📈 Budget" />
                <NavLink href="/reconcile" label="🔁 Rekonsiliasi" />
                <div class="mt-auto px-2">
                    <LogoutButton />
                </div>
            </aside>
            <main class="flex-1 p-6 max-w-6xl">{children()}</main>
        </div>
    }
}

#[component]
fn NavLink(href: &'static str, label: &'static str) -> impl IntoView {
    view! {
        <A href=href attr:class="block px-3 py-2 rounded hover:bg-slate-700">
            {label}
        </A>
    }
}

#[component]
fn LogoutButton() -> impl IntoView {
    let navigate = use_navigate();
    let on_click = move |_| {
        let navigate = navigate.clone();
        leptos::task::spawn_local(async move {
            let _ = crate::api::api_post_unit("/api/auth/logout", &serde_json::json!({})).await;
            navigate("/login", leptos_router::NavigateOptions::default());
        });
    };
    view! {
        <button
            on:click=on_click
            class="w-full text-left px-3 py-2 rounded bg-slate-800 hover:bg-red-700"
        >
            "🚪 Logout"
        </button>
    }
}
