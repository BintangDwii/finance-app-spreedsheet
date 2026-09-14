//! Rekening — CRUD + status (port `4_Accounts.py`).

use leptos::prelude::*;
use serde::Serialize;

#[derive(Serialize)]
struct CreateAccount {
    bank_name: String,
    no_rekening: String,
    currency: String,
    opening_balance: f64,
}

#[derive(Serialize)]
struct StatusBody {
    status: String,
}

#[component]
pub fn AccountsPage() -> impl IntoView {
    let list = LocalResource::new(|| async {
        crate::api::api_get::<serde_json::Value>("/api/accounts").await
    });
    let (bank, set_bank) = signal(String::new());
    let (norek, set_norek) = signal(String::new());
    let (saldo, set_saldo) = signal(String::new());

    let list_create = list;
    let on_create = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let body = CreateAccount {
            bank_name: bank.get(),
            no_rekening: norek.get(),
            currency: "IDR".to_string(),
            opening_balance: saldo.get().parse().unwrap_or(0.0),
        };
        leptos::task::spawn_local(async move {
            let _ =
                crate::api::api_post::<CreateAccount, serde_json::Value>("/api/accounts", &body)
                    .await;
            list_create.refetch();
        });
    };
    let set_status = move |id: i32, st: &'static str| {
        let body = StatusBody {
            status: st.to_string(),
        };
        leptos::task::spawn_local(async move {
            let _ = crate::api::api_post::<StatusBody, serde_json::Value>(
                &format!("/api/accounts/{id}/status"),
                &body,
            )
            .await;
            list.refetch();
        });
    };

    let field = "w-full border rounded px-3 py-2";
    view! {
        <h1 class="text-2xl font-bold mb-4">"🏦 Rekening"</h1>
        <Suspense fallback=move || view! { <p>"Memuat…"</p> }>
            {move || {
                list.get()
                    .map(|r| match r {
                        Err(e) => view! { <p class="text-red-600">{e}</p> }.into_any(),
                        Ok(v) => {
                            let rows: Vec<crate::api::Account> = v
                                .get("data")
                                .and_then(|d| serde_json::from_value(d.clone()).ok())
                                .unwrap_or_default();
                            let items = rows
                                .into_iter()
                                .map(|a| {
                                    let info = format!(
                                        "{} • {} • {} • {}",
                                        a.bank_name, a.no_rekening, a.currency, a.status,
                                    );
                                    view! {
                                        <li class="flex justify-between items-center border-b pb-1">
                                            <span>{info}</span>
                                            <span class="flex gap-1">
                                                <button
                                                    class="px-2 py-0.5 rounded bg-green-600 text-white text-xs"
                                                    on:click=move |_| set_status(a.id, "Aktif")
                                                >
                                                    "Aktif"
                                                </button>
                                                <button
                                                    class="px-2 py-0.5 rounded bg-amber-600 text-white text-xs"
                                                    on:click=move |_| set_status(a.id, "Beku")
                                                >
                                                    "Beku"
                                                </button>
                                            </span>
                                        </li>
                                    }
                                })
                                .collect_view();
                            view! {
                                <div class="bg-white rounded-xl shadow p-4 mb-4">
                                    <ul class="text-sm space-y-2">{items}</ul>
                                </div>
                            }
                                .into_any()
                        }
                    })
            }}
        </Suspense>
        <form on:submit=on_create class="bg-white rounded-xl shadow p-4 grid md:grid-cols-3 gap-3 max-w-3xl">
            <input class=field placeholder="Bank" on:input=move |ev| set_bank.set(event_target_value(&ev)) />
            <input class=field placeholder="No. Rekening" on:input=move |ev| set_norek.set(event_target_value(&ev)) />
            <input class=field placeholder="Saldo awal" on:input=move |ev| set_saldo.set(event_target_value(&ev)) />
            <button class="md:col-span-3 bg-slate-900 text-white rounded py-2">"Tambah Rekening"</button>
        </form>
    }
}
