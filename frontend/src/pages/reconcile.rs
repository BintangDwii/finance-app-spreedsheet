//! Rekonsiliasi — upload CSV + manual match (port `6_Reconcile.py`).
//! CSV dikirim sebagai text; server yang auto-match (§10: no client recompute).

use leptos::prelude::*;
use serde::Serialize;

#[derive(Serialize)]
struct MatchBody {
    transaksi_id: i64,
}

#[component]
pub fn ReconcilePage() -> impl IntoView {
    let list = LocalResource::new(|| async {
        crate::api::api_get::<serde_json::Value>("/api/reconcile").await
    });
    let (csv, set_csv) =
        signal("tanggal,deskripsi,amount\n2026-09-13,Transfer masuk,1500000\n".to_string());
    let (msg, set_msg) = signal(Option::<String>::None);
    let (match_id, set_match_id) = signal(String::new());

    let on_upload = move |_| {
        // Read once, reuse.
        let body = csv.get();
        leptos::task::spawn_local(async move {
            let client = reqwest::Client::new();
            let res = client
                .post("/api/reconcile/upload")
                .header("Content-Type", "text/csv")
                .body(body)
                .send()
                .await;
            match res {
                Ok(r) if r.status().is_success() => {
                    set_msg.set(Some("Upload OK — auto-match dijalankan server".to_string()));
                    list.refetch();
                }
                Ok(r) => {
                    set_msg.set(Some(format!(
                        "Gagal: {}",
                        r.text().await.unwrap_or_default()
                    )));
                }
                Err(e) => set_msg.set(Some(format!("Koneksi gagal: {e}"))),
            }
        });
    };

    let on_match = move |recon_id: i32| {
        let tid: i64 = match_id.get().parse().unwrap_or(0);
        let body = MatchBody { transaksi_id: tid };
        leptos::task::spawn_local(async move {
            let _ = crate::api::api_post::<MatchBody, serde_json::Value>(
                &format!("/api/reconcile/{recon_id}/match"),
                &body,
            )
            .await;
            list.refetch();
        });
    };

    view! {
        <h1 class="text-2xl font-bold mb-4">"🔁 Rekonsiliasi"</h1>
        {move || msg.get().map(|m| view! { <p class="text-sm mb-2">{m}</p> })}
        <div class="bg-white rounded-xl shadow p-4 max-w-3xl mb-4">
            <div class="font-semibold mb-2">"Upload Mutasi (CSV)"</div>
            <textarea
                class="w-full border rounded px-3 py-2 font-mono text-xs"
                rows="5"
                prop:value=move || csv.get()
                on:input=move |ev| set_csv.set(event_target_value(&ev))
            ></textarea>
            <button on:click=on_upload class="mt-2 bg-slate-900 text-white rounded px-4 py-2">
                "Upload & Auto-Match"
            </button>
        </div>
        <div class="bg-white rounded-xl shadow p-4 max-w-3xl mb-4">
            <label class="text-sm">
                "Transaksi ID untuk manual match"
                <input
                    class="w-full border rounded px-3 py-2"
                    on:input=move |ev| set_match_id.set(event_target_value(&ev))
                />
            </label>
        </div>
        <Suspense fallback=move || view! { <p>"Memuat…"</p> }>
            {move || {
                list.get()
                    .map(|r| match r {
                        Err(e) => view! { <p class="text-red-600">{e}</p> }.into_any(),
                        Ok(v) => {
                            let rows: Vec<crate::api::ReconRow> = v
                                .get("data")
                                .and_then(|d| serde_json::from_value(d.clone()).ok())
                                .unwrap_or_default();
                            let items = rows
                                .into_iter()
                                .map(|r| {
                                    let desc = r.description.unwrap_or_default();
                                    let amt = shared::format::format_idr(r.amount);
                                    view! {
                                        <tr class="border-b">
                                            <td class="p-2">{r.id}</td>
                                            <td class="p-2">{r.statement_date}</td>
                                            <td class="p-2">{desc}</td>
                                            <td class="p-2 text-right">{amt}</td>
                                            <td class="p-2">{r.status}</td>
                                            <td class="p-2">
                                                <button
                                                    class="px-2 py-0.5 rounded bg-blue-600 text-white text-xs"
                                                    on:click=move |_| on_match(r.id)
                                                >
                                                    "Match"
                                                </button>
                                            </td>
                                        </tr>
                                    }
                                })
                                .collect_view();
                            view! {
                                <div class="bg-white rounded-xl shadow overflow-x-auto">
                                    <table class="w-full text-sm">
                                        <thead>
                                            <tr class="text-left border-b">
                                                <th class="p-2">"ID"</th>
                                                <th class="p-2">"Tanggal"</th>
                                                <th class="p-2">"Deskripsi"</th>
                                                <th class="p-2 text-right">"Amount"</th>
                                                <th class="p-2">"Status"</th>
                                                <th class="p-2">"Aksi"</th>
                                            </tr>
                                        </thead>
                                        <tbody>{items}</tbody>
                                    </table>
                                </div>
                            }
                                .into_any()
                        }
                    })
            }}
        </Suspense>
    }
}
