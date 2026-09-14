//! Native SVG charts — zero JS dependency (replaces plotly).
//! Pure functions of API data; server aggregates stay authoritative.

use leptos::prelude::*;

/// Grouped bars: planned vs actual per `bulan/kategori` (top 12 by planned).
#[allow(unused_parens)] // `y=(expr)` is required view! macro syntax
#[component]
pub fn BudgetVarianceChart(rows: Vec<crate::api::VarianceRow>) -> impl IntoView {
    let mut top = rows.clone();
    top.sort_by(|a, b| {
        b.planned
            .partial_cmp(&a.planned)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    top.truncate(12);
    let max = top
        .iter()
        .map(|r| r.planned.max(r.actual))
        .fold(1.0_f64, f64::max);
    let bar = move |v: f64| format!("{:.1}", v / max * 280.0);
    let items = top
        .into_iter()
        .enumerate()
        .map(|(i, r)| {
            let y = 10 + i as i32 * 26;
            let wp = bar(r.planned);
            let wa = bar(r.actual);
            let fill = if r.actual > r.planned {
                "#f87171"
            } else {
                "#34d399"
            };
            let caption = format!("{} {}", r.bulan, r.kategori);
            view! {
                <g>
                    <text x="0" y=(y + 10) font-size="9" fill="#64748b">
                        {caption}
                    </text>
                    <rect x="150" y=y width=wp height="9" fill="#93c5fd" rx="2" />
                    <rect x="150" y=(y + 11) width=wa height="9" fill=fill rx="2" />
                </g>
            }
        })
        .collect_view();
    view! {
        <div class="bg-white rounded-xl shadow p-4">
            <div class="font-semibold mb-2">"Planned vs Actual (Top 12)"</div>
            <svg viewBox="0 0 600 320" class="w-full">
                {items}
            </svg>
            <div class="text-xs text-slate-500 flex gap-4">
                <span>"🟦 planned"</span>
                <span>"🟩 actual ≤ planned"</span>
                <span>"🟥 over budget"</span>
            </div>
        </div>
    }
}

/// Donut: pengeluaran per divisi from already-fetched rows (no extra request).
#[component]
pub fn DonutDivisi(rows: Vec<crate::api::Transaksi>) -> impl IntoView {
    use std::collections::BTreeMap;
    let mut map: BTreeMap<String, f64> = BTreeMap::new();
    for t in rows.iter().filter(|t| t.jenis == "Pengeluaran") {
        *map.entry(t.divisi.clone()).or_default() += t.idr_amount.unwrap_or(t.total_akhir);
    }
    let total: f64 = map.values().sum();
    let colors = [
        "#60a5fa", "#34d399", "#fbbf24", "#f87171", "#a78bfa", "#22d3ee", "#fb7185",
    ];
    // Build arcs once (§Function Reuse Rules).
    let mut acc = 0.0;
    let segs: Vec<(String, f64, f64, &'static str)> = map
        .into_iter()
        .enumerate()
        .map(|(i, (k, v))| {
            let denom = total.max(1.0);
            let a0 = acc / denom * 360.0;
            acc += v;
            let a1 = acc / denom * 360.0;
            (k, a0, a1, colors[i % colors.len()])
        })
        .collect();
    let paths = segs
        .iter()
        .map(|(_, a0, a1, c)| {
            let large = if a1 - a0 > 180.0 { 1 } else { 0 };
            let pt = |a: f64| {
                let r = a.to_radians();
                (60.0 + 45.0 * r.cos(), 60.0 + 45.0 * r.sin())
            };
            let (x0, y0) = pt(*a0 - 90.0);
            let (x1, y1) = pt(*a1 - 90.0);
            view! {
                <path
                    d=format!("M60,60 L{x0:.1},{y0:.1} A45,45 0 {large},1 {x1:.1},{y1:.1} Z")
                    fill=*c
                    opacity="0.85"
                />
            }
        })
        .collect_view();
    let legend = segs
        .into_iter()
        .map(|(k, _, _, c)| {
            view! {
                <li>
                    <span class="inline-block w-3 h-3 rounded-sm mr-1" style=format!("background:{c}")></span>
                    {k}
                </li>
            }
        })
        .collect_view();
    view! {
        <div class="bg-white rounded-xl shadow p-4">
            <div class="font-semibold mb-2">"Pengeluaran per Divisi"</div>
            <div class="flex items-center gap-4">
                <svg viewBox="0 0 120 120" class="w-36 h-36">
                    {paths}
                    <circle cx="60" cy="60" r="24" fill="white" />
                </svg>
                <ul class="text-xs space-y-1">{legend}</ul>
            </div>
        </div>
    }
}
