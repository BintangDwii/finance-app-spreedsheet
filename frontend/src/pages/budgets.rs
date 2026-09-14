//! Budget — variance + upsert (port `5_Budgets.py`).
//! Server `variance` is authoritative; no client-side recompute (§10).

use leptos::prelude::*;
use serde::Serialize;

#[derive(Serialize)]
struct UpsertBody {
    bulan: String,
    divisi: String,
    kategori: String,
    planned: f64,
}

#[component]
pub fn BudgetsPage() -> impl IntoView {
    let variance = LocalResource::new(|| async {
        crate::api::api_get::<serde_json::Value>("/api/budgets/variance").await
    });
    let (bulan, set_bulan) = signal("2026-09".to_string());
    let (divisi, set_divisi) = signal(String::new());
    let (kategori, set_kategori) = signal(String::new());
    let (planned, set_planned) = signal(String::new());

    let on_upsert = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let body = UpsertBody {
            bulan: bulan.get(),
            divisi: divisi.get(),
            kategori: kategori.get(),
            planned: planned.get().parse().unwrap_or(0.0),
        };
        leptos::task::spawn_local(async move {
            let _ =
                crate::api::api_post::<UpsertBody, serde_json::Value>("/api/budgets", &body).await;
            variance.refetch();
        });
    };

    let field = "w-full border rounded px-3 py-2";
    view! {
        <h1 class="text-2xl font-bold mb-4">"📈 Budget"</h1>
        <Suspense fallback=move || view! { <p>"Memuat variance…"</p> }>
            {move || {
                variance
                    .get()
                    .map(|r| match r {
                        Err(e) => view! { <p class="text-red-600">{e}</p> }.into_any(),
                        Ok(v) => {
                            let rows: Vec<crate::api::VarianceRow> = v
                                .get("data")
                                .and_then(|d| serde_json::from_value(d.clone()).ok())
                                .unwrap_or_default();
                            let table_rows = rows
                                .iter()
                                .map(|r| {
                                    let badge = if r.burn >= 100.0 {
                                        "🔴"
                                    } else if r.burn >= 80.0 {
                                        "🟡"
                                    } else {
                                        "🟢"
                                    };
                                    let burn = format!("{badge} {:.1}%", r.burn);
                                    view! {
                                        <tr class="border-b">
                                            <td class="p-2">{r.bulan.clone()}</td>
                                            <td class="p-2">{r.divisi.clone()}</td>
                                            <td class="p-2">{r.kategori.clone()}</td>
                                            <td class="p-2 text-right">
                                                {shared::format::format_idr(r.planned)}
                                            </td>
                                            <td class="p-2 text-right">
                                                {shared::format::format_idr(r.actual)}
                                            </td>
                                            <td class="p-2 text-right">
                                                {shared::format::format_idr(r.sisa)}
                                            </td>
                                            <td class="p-2 text-right">{burn}</td>
                                        </tr>
                                    }
                                })
                                .collect_view();
                            view! {
                                <div>
                                    <crate::components::charts::BudgetVarianceChart rows=rows />
                                    <div class="bg-white rounded-xl shadow overflow-x-auto mt-4">
                                        <table class="w-full text-sm">
                                            <thead>
                                                <tr class="text-left border-b">
                                                    <th class="p-2">"Bulan"</th>
                                                    <th class="p-2">"Divisi"</th>
                                                    <th class="p-2">"Kategori"</th>
                                                    <th class="p-2 text-right">"Planned"</th>
                                                    <th class="p-2 text-right">"Actual"</th>
                                                    <th class="p-2 text-right">"Sisa"</th>
                                                    <th class="p-2 text-right">"Burn%"</th>
                                                </tr>
                                            </thead>
                                            <tbody>{table_rows}</tbody>
                                        </table>
                                    </div>
                                </div>
                            }
                                .into_any()
                        }
                    })
            }}
        </Suspense>
        <h2 class="font-semibold mt-6 mb-2">"Upsert Budget (Admin/Approver)"</h2>
        <form on:submit=on_upsert class="bg-white rounded-xl shadow p-4 grid md:grid-cols-4 gap-3 max-w-4xl">
            <input
                class=field
                prop:value=move || bulan.get()
                on:input=move |ev| set_bulan.set(event_target_value(&ev))
            />
            <input class=field placeholder="Divisi" on:input=move |ev| set_divisi.set(event_target_value(&ev)) />
            <input class=field placeholder="Kategori" on:input=move |ev| set_kategori.set(event_target_value(&ev)) />
            <input class=field placeholder="Planned" on:input=move |ev| set_planned.set(event_target_value(&ev)) />
            <button class="md:col-span-4 bg-slate-900 text-white rounded py-2">"Simpan"</button>
        </form>
    }
}
