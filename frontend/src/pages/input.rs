//! Input transaksi — Maker/Admin (port `2_Input.py`).

use leptos::prelude::*;
use serde::Serialize;

#[derive(Serialize, Clone)]
struct CreateBody {
    tanggal: String,
    jenis: String,
    divisi: String,
    kategori: String,
    entitas_terkait: String,
    akun_pembayaran: String,
    akun_tujuan: Option<String>,
    subtotal: f64,
    ppn_11: Option<f64>,
    diskon: Option<f64>,
    currency: Option<String>,
    fx_rate: Option<f64>,
    due_date: Option<String>,
    reference_no: Option<String>,
    catatan: Option<String>,
}

#[component]
pub fn InputPage() -> impl IntoView {
    let (jenis, set_jenis) = signal("Pengeluaran".to_string());
    let (tanggal, set_tanggal) = signal("2026-09-14".to_string());
    let (divisi, set_divisi) = signal(String::new());
    let (kategori, set_kategori) = signal(String::new());
    let (entitas, set_entitas) = signal(String::new());
    let (akun, set_akun) = signal(String::new());
    let (tujuan, set_tujuan) = signal(String::new());
    let (subtotal, set_subtotal) = signal(String::new());
    let (catatan, set_catatan) = signal(String::new());
    let (msg, set_msg) = signal(Option::<String>::None);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        // Read once, reuse below.
        let kind = jenis.get();
        let body = CreateBody {
            tanggal: tanggal.get(),
            jenis: kind.clone(),
            divisi: divisi.get(),
            kategori: kategori.get(),
            entitas_terkait: entitas.get(),
            akun_pembayaran: akun.get(),
            akun_tujuan: (kind == "Transfer Internal").then(|| tujuan.get()),
            subtotal: subtotal.get().parse().unwrap_or(0.0),
            ppn_11: None,
            diskon: None,
            currency: Some("IDR".to_string()),
            fx_rate: None,
            due_date: None,
            reference_no: None,
            catatan: Some(catatan.get()),
        };
        leptos::task::spawn_local(async move {
            match crate::api::api_post::<CreateBody, serde_json::Value>("/api/transaksi", &body)
                .await
            {
                Ok(v) => set_msg.set(Some(format!("Tersimpan: {v}"))),
                Err(e) => set_msg.set(Some(format!("Gagal: {e}"))),
            }
        });
    };

    let field = "w-full border rounded px-3 py-2";
    view! {
        <h1 class="text-2xl font-bold mb-4">"➕ Input Transaksi"</h1>
        {move || msg.get().map(|m| view! { <p class="mb-2 text-sm">{m}</p> })}
        <form on:submit=on_submit class="bg-white rounded-xl shadow p-4 grid md:grid-cols-2 gap-3 max-w-3xl">
            <label class="text-sm">
                "Jenis"
                <select class=field on:change=move |ev| set_jenis.set(event_target_value(&ev))>
                    <option>"Pemasukan"</option>
                    <option selected>"Pengeluaran"</option>
                    <option>"Transfer Internal"</option>
                </select>
            </label>
            <label class="text-sm">
                "Tanggal (YYYY-MM-DD)"
                <input
                    class=field
                    prop:value=move || tanggal.get()
                    on:input=move |ev| set_tanggal.set(event_target_value(&ev))
                />
            </label>
            <label class="text-sm">
                "Divisi"
                <input class=field on:input=move |ev| set_divisi.set(event_target_value(&ev)) />
            </label>
            <label class="text-sm">
                "Kategori"
                <input class=field on:input=move |ev| set_kategori.set(event_target_value(&ev)) />
            </label>
            <label class="text-sm">
                "Entitas Terkait"
                <input class=field on:input=move |ev| set_entitas.set(event_target_value(&ev)) />
            </label>
            <label class="text-sm">
                "Akun Pembayaran"
                <input class=field on:input=move |ev| set_akun.set(event_target_value(&ev)) />
            </label>
            <Show when=move || jenis.get() == "Transfer Internal">
                <label class="text-sm">
                    "Akun Tujuan"
                    <input class=field on:input=move |ev| set_tujuan.set(event_target_value(&ev)) />
                </label>
            </Show>
            <label class="text-sm">
                "Subtotal"
                <input
                    class=field
                    inputmode="decimal"
                    on:input=move |ev| set_subtotal.set(event_target_value(&ev))
                />
            </label>
            <label class="text-sm md:col-span-2">
                "Catatan"
                <input class=field on:input=move |ev| set_catatan.set(event_target_value(&ev)) />
            </label>
            <button class="md:col-span-2 bg-slate-900 text-white rounded py-2">"Simpan"</button>
        </form>
    }
}
