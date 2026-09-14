//! Router + auth gate. `/login` public; rest requires `GET /api/auth/me`
//! (cookie session). 401 anywhere → back to login.

use leptos::prelude::*;
use leptos_router::{
    components::{Route, Router, Routes, A},
    hooks::use_navigate,
    path, NavigateOptions,
};

use crate::{api, components::layout::Layout, pages};

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <Routes fallback=|| view! { <NotFound /> }>
                <Route path=path!("/login") view=pages::login::LoginPage />
                <Route path=path!("/") view=move || gate(|| view! { <pages::dashboard::DashboardPage /> }.into_any()) />
                <Route path=path!("/input") view=move || gate(|| view! { <pages::input::InputPage /> }.into_any()) />
                <Route path=path!("/approval") view=move || gate(|| view! { <pages::approval::ApprovalPage /> }.into_any()) />
                <Route path=path!("/accounts") view=move || gate(|| view! { <pages::accounts::AccountsPage /> }.into_any()) />
                <Route path=path!("/budgets") view=move || gate(|| view! { <pages::budgets::BudgetsPage /> }.into_any()) />
                <Route path=path!("/reconcile") view=move || gate(|| view! { <pages::reconcile::ReconcilePage /> }.into_any()) />
            </Routes>
        </Router>
    }
}

/// Session gate: loads `/api/auth/me` once per page, renders `Layout` +
/// page on success, redirects to `/login` on 401. Takes a `Clone` page
/// factory (not `Children`) because `Children` is `FnOnce` and cannot be
/// re-invoked from the reactive render closure.
fn gate(page: impl Fn() -> AnyView + Clone + Send + Sync + 'static) -> impl IntoView {
    let navigate = use_navigate();
    let page = StoredValue::new(page);
    let me = LocalResource::new(|| async { api::api_get::<api::MeResponse>("/api/auth/me").await });
    Effect::new(move || {
        if matches!(me.get(), Some(Err(_))) {
            navigate("/login", NavigateOptions::default());
        }
    });
    view! {
        <Suspense fallback=move || view! { <p class="p-6">"Memuat sesi…"</p> }>
            {move || {
                me.get()
                    .and_then(|r| r.ok())
                    .map(|m| {
                        let user = api::UserPublic {
                            username: m.username,
                            role: m.role,
                            divisi: m.divisi,
                        };
                        view! { <Layout user=user>{(page.get_value())()}</Layout> }
                    })
            }}
        </Suspense>
    }
}

#[component]
fn NotFound() -> impl IntoView {
    view! {
        <p class="p-6">
            "404 — halaman tidak ada. " <A href="/" attr:class="underline">"Kembali"</A>
        </p>
    }
}
