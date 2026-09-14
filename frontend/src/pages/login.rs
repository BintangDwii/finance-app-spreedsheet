//! Login — `POST /api/auth/login` sets HttpOnly cookie + returns user.

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use serde::Serialize;

#[derive(Serialize)]
struct LoginBody {
    username: String,
    password: String,
}

#[component]
pub fn LoginPage() -> impl IntoView {
    let (username, set_username) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (err, set_err) = signal(Option::<String>::None);
    let navigate = use_navigate();

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let navigate = navigate.clone();
        // Read once, reuse (§Function Reuse Rules).
        let body = LoginBody {
            username: username.get(),
            password: password.get(),
        };
        leptos::task::spawn_local(async move {
            match crate::api::api_post::<LoginBody, crate::api::LoginResponse>(
                "/api/auth/login",
                &body,
            )
            .await
            {
                Ok(_) => navigate("/", Default::default()),
                Err(e) => set_err.set(Some(e)),
            }
        });
    };

    view! {
        <div class="min-h-screen flex items-center justify-center bg-slate-900">
            <form on:submit=on_submit class="bg-white rounded-xl p-8 w-96 space-y-4 shadow-xl">
                <h1 class="text-xl font-bold">"🏢 Corporate Finance"</h1>
                <p class="text-sm text-slate-500">"Masuk untuk melanjutkan"</p>
                {move || err.get().map(|e| view! { <p class="text-sm text-red-600">{e}</p> })}
                <input
                    class="w-full border rounded px-3 py-2"
                    placeholder="Username"
                    prop:value=move || username.get()
                    on:input=move |ev| set_username.set(event_target_value(&ev))
                />
                <input
                    class="w-full border rounded px-3 py-2"
                    type="password"
                    placeholder="Password"
                    prop:value=move || password.get()
                    on:input=move |ev| set_password.set(event_target_value(&ev))
                />
                <button class="w-full bg-slate-900 text-white rounded py-2 hover:bg-slate-700">
                    "Login"
                </button>
            </form>
        </div>
    }
}
