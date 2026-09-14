//! Typed same-origin API client (§2: handlers validate, this only shapes JSON).
//! One `api_get`/`api_post` pair reused by all pages (§10 DRY).

use serde::{de::DeserializeOwned, Serialize};

fn client() -> reqwest::Client {
    reqwest::Client::new()
}

async fn check_unauthorized(res: &reqwest::Response) -> bool {
    res.status() == reqwest::StatusCode::UNAUTHORIZED
}

pub async fn api_get<T: DeserializeOwned>(path: &str) -> Result<T, String> {
    let res = client()
        .get(path)
        .send()
        .await
        .map_err(|e| format!("Koneksi gagal: {e}"))?;
    if check_unauthorized(&res).await {
        return Err("UNAUTHORIZED".into());
    }
    res.json::<T>()
        .await
        .map_err(|e| format!("Respon tak valid: {e}"))
}

pub async fn api_post<B: Serialize, T: DeserializeOwned>(
    path: &str,
    body: &B,
) -> Result<T, String> {
    let res = client()
        .post(path)
        .json(body)
        .send()
        .await
        .map_err(|e| format!("Koneksi gagal: {e}"))?;
    if check_unauthorized(&res).await {
        return Err("UNAUTHORIZED".into());
    }
    if !res.status().is_success() {
        let msg = res.text().await.unwrap_or_else(|_| "unknown".into());
        return Err(msg);
    }
    res.json::<T>()
        .await
        .map_err(|e| format!("Respon tak valid: {e}"))
}

pub async fn api_post_unit<B: Serialize>(path: &str, body: &B) -> Result<(), String> {
    let res = client()
        .post(path)
        .json(body)
        .send()
        .await
        .map_err(|e| format!("Koneksi gagal: {e}"))?;
    if check_unauthorized(&res).await {
        return Err("UNAUTHORIZED".into());
    }
    if !res.status().is_success() {
        let msg = res.text().await.unwrap_or_else(|_| "unknown".into());
        return Err(msg);
    }
    Ok(())
}

pub async fn api_delete(path: &str) -> Result<(), String> {
    let res = client()
        .delete(path)
        .send()
        .await
        .map_err(|e| format!("Koneksi gagal: {e}"))?;
    if check_unauthorized(&res).await {
        return Err("UNAUTHORIZED".into());
    }
    if !res.status().is_success() {
        let msg = res.text().await.unwrap_or_else(|_| "unknown".into());
        return Err(msg);
    }
    Ok(())
}

pub async fn api_upload_bukti(id: i64, file: web_sys::File) -> Result<serde_json::Value, String> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    let form_data = web_sys::FormData::new().map_err(|_| "Gagal buat FormData".to_string())?;
    form_data
        .append_with_blob_and_filename("file", &file, &file.name())
        .map_err(|_| "Gagal attach file".to_string())?;

    let window = web_sys::window().ok_or_else(|| "No window".to_string())?;
    let opts = web_sys::RequestInit::new();
    opts.set_method("POST");
    opts.set_body(&form_data);

    let request =
        web_sys::Request::new_with_str_and_init(&format!("/api/transaksi/{id}/bukti"), &opts)
            .map_err(|_| "Gagal buat request".to_string())?;

    let resp_val = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|_| "Koneksi gagal".to_string())?;

    let resp: web_sys::Response = resp_val
        .dyn_into()
        .map_err(|_| "Response type error".to_string())?;

    if resp.status() == 401 {
        return Err("UNAUTHORIZED".into());
    }

    let text_js = JsFuture::from(resp.text().map_err(|_| "Read text error".to_string())?)
        .await
        .map_err(|_| "Text promise error".to_string())?;

    let text = text_js.as_string().unwrap_or_default();
    if !resp.ok() {
        return Err(text);
    }

    serde_json::from_str(&text).map_err(|e| format!("Respon JSON tak valid: {e}"))
}

// ---------- Shared response shapes (mirror backend JSON) ----------

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct UserPublic {
    pub username: String,
    pub role: shared::domain::UserRole,
    pub divisi: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserPublic,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct MeResponse {
    pub username: String,
    pub role: shared::domain::UserRole,
    pub divisi: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Transaksi {
    pub id: i64,
    pub tanggal: String,
    pub jenis: String,
    pub divisi: String,
    pub kategori: String,
    pub entitas_terkait: String,
    pub akun_pembayaran: String,
    pub subtotal: f64,
    pub ppn_11: f64,
    pub diskon: f64,
    pub total_akhir: f64,
    pub currency: String,
    pub fx_rate: f64,
    pub reference_no: Option<String>,
    pub status: String,
    pub file_bukti: Option<String>,
    pub catatan: Option<String>,
    pub idr_amount: Option<f64>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct TransaksiList {
    #[serde(default)]
    pub data: Vec<Transaksi>,
    #[serde(default)]
    pub page: i64,
    #[serde(default)]
    pub per_page: i64,
    #[serde(default)]
    pub total: i64,
    #[serde(default)]
    pub total_pages: i64,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Saldo {
    #[serde(default)]
    pub masuk: f64,
    #[serde(default)]
    pub keluar: f64,
    #[serde(default)]
    pub saldo: f64,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct RekeningSaldo {
    pub bank_name: String,
    pub currency: String,
    #[serde(default)]
    pub opening_balance: f64,
    #[serde(default)]
    pub total_masuk: f64,
    #[serde(default)]
    pub total_keluar: f64,
    #[serde(default)]
    pub saldo: f64,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct VarianceRow {
    pub bulan: String,
    pub divisi: String,
    pub kategori: String,
    #[serde(default)]
    pub planned: f64,
    #[serde(default)]
    pub actual: f64,
    #[serde(default)]
    pub sisa: f64,
    #[serde(default)]
    pub burn: f64,
}

/// Raw budget row (`GET /api/budgets`) — kept for future CRUD UI.
#[allow(dead_code)]
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Budget {
    pub id: i32,
    pub bulan: String,
    pub divisi: String,
    pub kategori: String,
    #[serde(default)]
    pub planned: f64,
    pub currency: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ReconRow {
    pub id: i32,
    pub statement_date: String,
    pub description: Option<String>,
    #[serde(default)]
    pub amount: f64,
    pub currency: String,
    pub matched_transaksi_id: Option<i64>,
    pub status: String,
    pub uploaded_by: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Account {
    pub id: i32,
    pub bank_name: String,
    pub no_rekening: String,
    pub currency: String,
    #[serde(default)]
    pub opening_balance: f64,
    pub status: String,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct AuditRow {
    pub id: i32,
    pub transaksi_id: i64,
    pub actor: String,
    pub from_status: Option<String>,
    pub to_status: Option<String>,
    pub catatan: Option<String>,
}
