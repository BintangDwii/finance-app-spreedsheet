//! Approval — list + filter + update status + soft-delete + audit (port `3_Approval.py`).

use leptos::prelude::*;
use serde::Serialize;

#[derive(Serialize)]
struct StatusBody {
    new_status: String,
    catatan: Option<String>,
}

const FILTERS: [&str; 5] = [
    "",
    "Pending (Butuh Approval)",
    "Approved (Menunggu Bayar)",
    "Paid (Lunas)",
    "Rejected (Ditolak)",
];

#[component]
pub fn ApprovalPage() -> impl IntoView {
    let (status, set_status) = signal(String::new());
    // LocalResource tracks `status` reads and refetches automatically; mutations
    // call `.refetch()` explicitly (no version counter needed).
    let list = LocalResource::new(move || async move {
        let st = status.get();
        let q = if st.is_empty() {
            "/api/transaksi?per_page=50".to_string()
        } else {
            format!("/api/transaksi?per_page=50&status={st}")
        };
        crate::api::api_get::<crate::api::TransaksiList>(&q).await
    });
    let audit = LocalResource::new(|| async {
        crate::api::api_get::<serde_json::Value>("/api/audit?limit=100").await
    });

    let act = move |id: i64, to: &'static str| {
        let body = StatusBody {
            new_status: to.to_string(),
            catatan: Some("via web".to_string()),
        };
        leptos::task::spawn_local(async move {
            let _ = crate::api::api_post::<StatusBody, serde_json::Value>(
                &format!("/api/transaksi/{id}/status"),
                &body,
            )
            .await;
            list.refetch();
            audit.refetch();
        });
    };
    let del = move |id: i64| {
        leptos::task::spawn_local(async move {
            let _ = crate::api::api_delete(&format!("/api/transaksi/{id}")).await;
            list.refetch();
        });
    };
    let upload = move |id: i64, ev: leptos::ev::Event| {
        use wasm_bindgen::JsCast;
        let target: web_sys::HtmlInputElement = ev.target().unwrap().unchecked_into();
        if let Some(files) = target.files() {
            if let Some(file) = files.get(0) {
                leptos::task::spawn_local(async move {
                    let _ = crate::api::api_upload_bukti(id, file).await;
                    list.refetch();
                });
            }
        }
    };

    view! {
        <h1 class="text-2xl font-bold mb-4">"✅ Approval"</h1>
        <div class="flex gap-2 mb-3 flex-wrap">
            {FILTERS
                .into_iter()
                .map(|s| {
                    let label = if s.is_empty() { "Semua" } else { s };
                    view! {
                        <button
                            class="px-3 py-1 rounded bg-white shadow text-sm hover:bg-slate-200"
                            on:click=move |_| set_status.set(s.to_string())
                        >
                            {label}
                        </button>
                    }
                })
                .collect_view()}
        </div>
        <Suspense fallback=move || view! { <p>"Memuat…"</p> }>
            {move || {
                list.get()
                    .map(|r| match r {
                        Err(e) => view! { <p class="text-red-600">{e}</p> }.into_any(),
                        Ok(l) => {
                            let summary = format!(
                                "Hal {} / {} • {} data",
                                l.page.max(1),
                                l.total_pages.max(1),
                                l.total,
                            );
                            let rows = l
                                .data
                                .into_iter()
                                .map(|t| {
                                    let amt = shared::format::format_idr(
                                        t.idr_amount.unwrap_or(t.total_akhir),
                                    );
                                    let bukti_view = match t.file_bukti.as_deref() {
                                        Some(f) if !f.is_empty() && f != "-" => {
                                            view! {
                                                <a
                                                    href=format!("/uploads/{f}")
                                                    target="_blank"
                                                    class="text-blue-600 underline text-xs block truncate max-w-[120px]"
                                                >
                                                    "📄 " {f}
                                                </a>
                                            }
                                                .into_any()
                                        }
                                        _ => view! { <span class="text-xs text-slate-400">"-"</span> }.into_any(),
                                    };
                                    view! {
                                        <tr class="border-b hover:bg-slate-50">
                                            <td class="p-2">{t.id}</td>
                                            <td class="p-2">{t.tanggal}</td>
                                            <td class="p-2">{t.jenis}</td>
                                            <td class="p-2">{t.entitas_terkait}</td>
                                            <td class="p-2 text-right">{amt}</td>
                                            <td class="p-2">{t.status}</td>
                                            <td class="p-2">{bukti_view}</td>
                                            <td class="p-2 flex gap-1 items-center">
                                                <button
                                                    class="px-2 py-0.5 rounded bg-green-600 text-white text-xs"
                                                    on:click=move |_| act(t.id, "Approved (Menunggu Bayar)")
                                                >
                                                    "Approve"
                                                </button>
                                                <button
                                                    class="px-2 py-0.5 rounded bg-blue-600 text-white text-xs"
                                                    on:click=move |_| act(t.id, "Paid (Lunas)")
                                                >
                                                    "Paid"
                                                </button>
                                                <button
                                                    class="px-2 py-0.5 rounded bg-red-600 text-white text-xs"
                                                    on:click=move |_| act(t.id, "Rejected (Ditolak)")
                                                >
                                                    "Reject"
                                                </button>
                                                <label class="px-2 py-0.5 rounded bg-amber-600 text-white text-xs cursor-pointer">
                                                    "Upload"
                                                    <input
                                                        type="file"
                                                        accept=".pdf,.jpg,.jpeg,.png"
                                                        class="hidden"
                                                        on:change=move |ev| upload(t.id, ev)
                                                    />
                                                </label>
                                                <button
                                                    class="px-2 py-0.5 rounded bg-slate-500 text-white text-xs"
                                                    on:click=move |_| del(t.id)
                                                >
                                                    "Hapus"
                                                </button>
                                            </td>
                                        </tr>
                                    }
                                })
                                .collect_view();
                            view! {
                                <div>
                                    <p class="text-xs text-slate-500 mb-2">{summary}</p>
                                    <div class="bg-white rounded-xl shadow overflow-x-auto">
                                        <table class="w-full text-sm">
                                            <thead>
                                                <tr class="text-left border-b">
                                                    <th class="p-2">"ID"</th>
                                                    <th class="p-2">"Tanggal"</th>
                                                    <th class="p-2">"Jenis"</th>
                                                    <th class="p-2">"Entitas"</th>
                                                    <th class="p-2 text-right">"Total"</th>
                                                    <th class="p-2">"Status"</th>
                                                    <th class="p-2">"Bukti"</th>
                                                    <th class="p-2">"Aksi"</th>
                                                </tr>
                                            </thead>
                                            <tbody>{rows}</tbody>
                                        </table>
                                    </div>
                                </div>
                            }
                                .into_any()
                        }
                    })
            }}
        </Suspense>
        <h2 class="font-semibold mt-6 mb-2">"Audit Trail (100 terakhir)"</h2>
        <Suspense fallback=move || view! { <p>"Memuat audit…"</p> }>
            {move || {
                audit
                    .get()
                    .map(|r| match r {
                        Err(e) => view! { <p class="text-red-600">{e}</p> }.into_any(),
                        Ok(v) => {
                            let rows: Vec<crate::api::AuditRow> = v
                                .get("data")
                                .and_then(|d| serde_json::from_value(d.clone()).ok())
                                .unwrap_or_default();
                            let items = rows
                                .into_iter()
                                .map(|a| {
                                    let line = format!(
                                        "#{} trx:{} {}: {} → {} ({})",
                                        a.id,
                                        a.transaksi_id,
                                        a.actor,
                                        a.from_status.unwrap_or_default(),
                                        a.to_status.unwrap_or_default(),
                                        a.catatan.unwrap_or_default(),
                                    );
                                    view! { <li>{line}</li> }
                                })
                                .collect_view();
                            view! {
                                <ul class="text-xs space-y-1 bg-white rounded-xl shadow p-3">{items}</ul>
                            }
                                .into_any()
                        }
                    })
            }}
        </Suspense>
    }
}
