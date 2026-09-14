//! Dashboard — KPI + per-rekening + variance (port `1_Dashboard.py`).

use leptos::prelude::*;

use crate::{
    api,
    components::{
        cards::KpiRow,
        charts::{BudgetVarianceChart, DonutDivisi},
    },
};

fn parse_data<T: serde::de::DeserializeOwned>(v: serde_json::Value) -> Vec<T> {
    v.get("data")
        .and_then(|d| serde_json::from_value(d.clone()).ok())
        .unwrap_or_default()
}

#[component]
pub fn DashboardPage() -> impl IntoView {
    let saldo = LocalResource::new(|| async { api::api_get::<api::Saldo>("/api/saldo").await });
    let reks = LocalResource::new(|| async {
        api::api_get::<serde_json::Value>("/api/saldo/per-rekening").await
    });
    let variance = LocalResource::new(|| async {
        api::api_get::<serde_json::Value>("/api/budgets/variance").await
    });
    let recent = LocalResource::new(|| async {
        api::api_get::<api::TransaksiList>("/api/transaksi?per_page=200").await
    });

    view! {
        <h1 class="text-2xl font-bold mb-4">"📊 Dashboard"</h1>
        <Suspense fallback=move || view! { <p>"Memuat saldo…"</p> }>
            {move || {
                saldo
                    .get()
                    .map(|r| match r {
                        Ok(s) => {
                            view! {
                                <KpiRow masuk=s.masuk keluar=s.keluar saldo=s.saldo pending_count=0 />
                            }
                                .into_any()
                        }
                        Err(e) => view! { <p class="text-red-600">{e}</p> }.into_any(),
                    })
            }}
        </Suspense>

        <div class="grid md:grid-cols-2 gap-4 mt-4">
            <Suspense fallback=move || view! { <p>"Memuat rekening…"</p> }>
                {move || {
                    reks
                        .get()
                        .map(|r| match r {
                            Ok(v) => {
                                let rows: Vec<api::RekeningSaldo> = parse_data(v);
                                view! {
                                    <div class="bg-white rounded-xl shadow p-4">
                                        <div class="font-semibold mb-2">"Saldo per Rekening"</div>
                                        <ul class="text-sm space-y-1">
                                            {rows
                                                .into_iter()
                                                .map(|r| {
                                                    let name = format!("{} ({})", r.bank_name, r.currency);
                                                    let bal = shared::format::format_idr(r.saldo);
                                                    view! {
                                                        <li class="flex justify-between">
                                                            <span>{name}</span>
                                                            <b>{bal}</b>
                                                        </li>
                                                    }
                                                })
                                                .collect_view()}
                                        </ul>
                                    </div>
                                }
                                    .into_any()
                            }
                            Err(e) => view! { <p class="text-red-600">{e}</p> }.into_any(),
                        })
                }}
            </Suspense>
            <Suspense fallback=move || view! { <p>"Memuat variance…"</p> }>
                {move || {
                    variance
                        .get()
                        .map(|r| match r {
                            Ok(v) => {
                                let rows: Vec<api::VarianceRow> = parse_data(v);
                                view! { <BudgetVarianceChart rows=rows /> }.into_any()
                            }
                            Err(e) => view! { <p class="text-red-600">{e}</p> }.into_any(),
                        })
                }}
            </Suspense>
        </div>

        <div class="mt-4">
            <Suspense fallback=move || view! { <p>"Memuat transaksi…"</p> }>
                {move || {
                    recent
                        .get()
                        .map(|r| match r {
                            Ok(l) => view! { <DonutDivisi rows=l.data /> }.into_any(),
                            Err(e) => view! { <p class="text-red-600">{e}</p> }.into_any(),
                        })
                }}
            </Suspense>
        </div>
    }
}
