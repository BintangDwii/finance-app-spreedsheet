//! KPI cards — port of `components/cards.py`.

use leptos::prelude::*;

#[component]
pub fn MetricCard(
    label: String,
    value: String,
    #[prop(optional)] delta: Option<String>,
) -> impl IntoView {
    view! {
        <div class="bg-white rounded-xl shadow p-4">
            <div class="text-sm text-slate-500">{label}</div>
            <div class="text-2xl font-bold">{value}</div>
            {delta.map(|d| view! { <div class="text-xs text-slate-400">{d}</div> })}
        </div>
    }
}

#[component]
pub fn KpiRow(masuk: f64, keluar: f64, saldo: f64, pending_count: usize) -> impl IntoView {
    view! {
        <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
            <MetricCard label="Pemasukan".to_string() value=shared::format::format_idr(masuk) />
            <MetricCard label="Pengeluaran".to_string() value=shared::format::format_idr(keluar) />
            <MetricCard label="Saldo".to_string() value=shared::format::format_idr(saldo) />
            <MetricCard
                label="Pending".to_string()
                value=pending_count.to_string()
                delta="butuh approval".to_string()
            />
        </div>
    }
}
