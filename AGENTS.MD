
# Rust Agent Coding Standards (`AGENTS.MD`)

File ini mendefinisikan pedoman operasional dan standar arsitektur tingkat lanjut bagi AI Agent dalam menulis, memelihara, dan merefaktor kode sumber berbasis bahasa pemrograman Rust agar memenuhi standar industri (*production-ready*).

---

## 1. Type Safety, Enums, & The Newtype Pattern

AI **wajib** memaksimalkan sistem tipe statis Rust untuk mencegah *primitive obsession* (mengandalkan tipe dasar seperti `String`, `i32`, atau `bool` untuk merepresentasikan konsep domain spesifik).

* **Enums untuk State & Finite Options:** Hindari penggunaan `String` atau magic number untuk status atau kategori.
```rust
// ❌ DILARANG: Rawan typo dan tidak type-safe
fn update_user_role(role: &str) { ... }

// ✅ DIANJURKAN: Tipe terdefinisi dengan jelas
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum UserRole {
    Admin,
    Moderator,
    Member,
}

```


* **The Newtype Pattern:** Gunakan pembungkus tuple struct untuk mencegah pertukaran parameter antar ID primitif yang sama (misal `Uuid` untuk User vs Order).
```rust
// ✅ DIANJURKAN: Mencegah salah kirim ID antar fungsi
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, sqlx::Type)]
#[sqlx(transparent)]
pub struct UserId(pub uuid::Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, sqlx::Type)]
#[sqlx(transparent)]
pub struct OrderId(pub uuid::Uuid);

```


* **Exhaustive Pattern Matching:** Selalu tangani seluruh varian `enum` secara eksplisit menggunakan `match`. Hindari penggunaan wildcard `_` pada logika inti bisnis kecuali benar-benar diperlukan untuk ekspansi terbuka.

---

## 2. DRY, Modularity, & Clean Architecture

Struktur direktori proyek harus memisahkan tanggung jawab dengan tegas mengikuti prinsip *Clean Architecture* atau pemisahan *layers*.

* **Pemisahan Lapisan (Layering):**
* `handler / controller`: Menerima HTTP request, melakukan validasi payload awal, memanggil service.
* `service / usecase`: Berisi logika inti bisnis (*business rules*). Tidak boleh bergantung langsung pada framework web tertentu.
* `repository`: Menangani interaksi basis data (SQLx/Diesel). Mengabstraksi operasi penyimpanan data.


* **Reusability melalui Generics & Traits:** Jika ada perilaku serupa di berbagai entitas, definisikan `trait` bersama alih-alih menduplikasi fungsi implementasi.
```rust
// ✅ DIANJURKAN: Trait generik untuk operasi umum
#[async_trait::async_trait]
pub trait Repository<T, ID> {
    async fn find_by_id(&self, id: ID) -> Result<Option<T>, AppError>;
    async fn save(&self, entity: &T) -> Result<(), AppError>;
}

```



---

## 3. Robust Error Handling

* **Custom Error Enums:** Jangan pernah menggunakan `unwrap()` atau `expect()` pada *production code*. Gunakan pustaka `thiserror` untuk mendefinisikan error domain.
```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error occurred: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Resource not found: {entity} with id {id}")]
    NotFound { entity: &'static str, id: String },

    #[error("Invalid payload: {0}")]
    Validation(String),
}

```


* **Result Alias:** Definisikan alias tipe hasil di level modul untuk efisiensi penulisan:
```rust
pub type Result<T> = std::result::Result<T, AppError>;

```



---

## 4. Asynchronous Programming & State Management

* **Thread Safety (`Send + Sync`):** Pastikan semua struktur data yang dibagikan antar thread di dalam *async runtime* (seperti Tokio atau Actix Web) memenuhi syarat `Send + 'sync`.
* **Shared State Concurrency:** Gunakan `Arc<T>` bersama dengan `RwLock<T>` atau `Mutex<T>` secara hati-hati untuk manajemen state global. Hindari *deadlock* dengan menjaga durasi penguncian (*lock scope*) sesingkat mungkin.
* **Blocking Operations Offloading:** Operasi I/O sinkron atau komputasi berat wajib dipindahkan menggunakan `tokio::task::spawn_blocking` agar tidak memblokir *async event loop*.

---

## 5. Database & Persistence Layer Best Practices

* **Parameter Binding:** Semua kueri SQL wajib menggunakan *parameterized queries* (menggunakan `$1`, `$2` di SQLx) untuk mencegah kerentanan SQL Injection. Jangan pernah melakukan *string interpolation* pada kueri basis data.
* **Mapping Langsung ke Struct:** Petakan hasil basis data langsung ke dalam domain struct menggunakan tipe data yang aman (misal: `chrono::DateTime<Utc>`, `sqlx::types::Uuid`), hindari penggunaan tipe mentah di lapisan bisnis.
* **Connection Pooling:** Selalu gunakan *connection pool* yang dikelola framework (seperti `sqlx::PgPool`) dan bagikan melalui *application state*.

---

## 6. Observability, Logging, & Tracing

* **Structured Logging:** Gunakan pustaka `tracing` alih-alih `println!` atau *macro* `log` biasa untuk melacak alur eksekusi asinkron secara kontekstual.
```rust
#[tracing::instrument(skip(pool))]
pub async fn get_user_profile(pool: &PgPool, user_id: UserId) -> Result<UserProfile> {
    tracing::info!("Fetching user profile from database");
    // Implementasi logika...
}

```



---

## 7. Security & Input Validation

* **Validasi Payload:** Gunakan pustaka validasi (seperti `validator`) pada *DTO* (Data Transfer Object) sebelum data masuk ke lapisan *service* atau *repository*.
* **Sanitisasi Data:** Pastikan string eksternal atau input pengguna dibersihkan dari karakter berbahaya sebelum diproses lebih lanjut.

---

## 8. Testing Strategy

* **Unit Tests:** Setiap modul logika bisnis wajib disertai unit test di dalam file yang sama (`#[cfg(test)] mod tests`).
* **Integration Tests:** Letakkan tes integrasi di dalam direktori `tests/` terpisah untuk menguji alur end-to-end dari API atau interaksi basis data menggunakan basis data uji (misal testcontainers atau database transaksional terisolasi).

---

## 9. Code Quality Gates

* Kode yang dihasilkan wajib bebas dari peringatan compiler dan lolos pemeriksaan `cargo clippy` tanpa *warning*.
* Format kode wajib mematuhi standar penataan otomatis bawaan `cargo fmt` .


## 10. Reusability, Shared Utilities, & DRY Enforcement

AI **wajib** memastikan tidak ada duplikasi logika (*code duplication*) di dalam codebase. Setiap pola yang sering digunakan kembali harus di-ekstrak menjadi utilitas terpusat.

*   **Centralized Database Queries / Helpers:** Jangan pernah menulis ulang logika query SQL yang serupa di handler atau service yang berbeda. Bungkus operasi umum ke dalam *Repository* atau fungsi helper terpusat.
*   **Shared Validation & Formatting Utilities:** 
    *   Fungsi pemformatan (misal: konversi mata uang ke IDR, pemformatan tanggal, sanitasi string) wajib diletakkan di modul `src/utils/` atau `src/helpers/` yang dapat diimpor secara global.
    *   Jangan mendeklarasikan ulang fungsi helper di dalam file handler atau modul bisnis spesifik jika sifatnya generik.
*   **Generic Repository & Traits:** Manfaatkan*Traits* dan *Generics* untuk operasi CRUD standar agar satu implementasi trait dapat digunakan oleh berbagai entitas domain (User, Transaksi, Account, dll).


## Function Call & Reuse Rules

### 1. Prefer Call Once, Reuse Multiple Times

When the same function result is needed multiple times within the same scope:

* Call the function once near the top of the relevant scope.
* Store the result in a clearly named variable.
* Reuse that variable instead of calling the function repeatedly.

**Preferred:**

```rust
let employee = get_employee().await?;

show_name(&employee);
show_email(&employee);
show_role(&employee);
```

**Avoid:**

```rust
show_name(&get_employee().await?);
show_email(&get_employee().await?);
show_role(&get_employee().await?);
```

### 2. Do Not Repeatedly Call Expensive or Async Functions

Do not call the same expensive, asynchronous, database, API, or external-service function multiple times when the result can be reused.

**Preferred:**

```rust
let employees = get_employees().await?;

let total = employees.len();
let active = employees.iter().filter(|e| e.active).count();
let first = employees.first();
```

**Avoid:**

```rust
let total = get_employees().await?.len();
let active = get_employees().await?.iter().filter(|e| e.active).count();
let first = get_employees().await?.first();
```

### 3. Reuse Existing Variables

Before calling a function, check whether the required data has already been retrieved or calculated in the current scope.

Do not fetch or calculate the same data again if an existing variable can be reused.

### 4. Keep Reused Values Close to Their First Use

Do not create a large number of variables at the top of a function without a reason.

Place reusable values near the beginning of the logical section where they are needed.

**Preferred:**

```rust
let employee = get_employee().await?;

// Employee-related logic
...
```

Instead of unnecessarily collecting everything at the top:

```rust
let employee = get_employee().await?;
let payroll = get_payroll().await?;
let attendance = get_attendance().await?;
let settings = get_settings().await?;

// Only payroll is needed in this section.
```

### 5. Avoid Premature Abstraction

Do not create aliases, wrappers, closures, or helper functions only to shorten a single function call.

Create an abstraction when it improves:

* readability
* reuse
* maintainability
* separation of responsibility

Do not sacrifice clarity just to reduce the number of characters.

### 6. Be Careful With Mutable State

Only reuse a stored value when its state is still valid.

If a function can return different results because of state changes, user input, time, database changes, or side effects, do not automatically replace repeated calls with one cached result.

### 7. Database and API Calls

Database queries and external API calls should generally be performed once when the same result is required by multiple operations.

Prefer:

```rust
let employee = repository.find_by_id(id).await?;

let name = employee.name;
let role = employee.role;
let status = employee.status;
```

Avoid unnecessarily repeating:

```rust
repository.find_by_id(id).await?;
repository.find_by_id(id).await?;
repository.find_by_id(id).await?;
```

unless fresh data is intentionally required.

### 8. Do Not Optimize Blindly

Function reuse is a readability and efficiency guideline, not an absolute rule.

Do not introduce caching or reuse when:

* the function intentionally has side effects
* fresh data is required
* the function is trivial and repeated once or twice for readability
* ownership/borrowing would make the code significantly more complicated
* the resulting code becomes harder to understand

### 9. Prefer Clear Naming

When storing a function result for reuse, use descriptive names.

**Preferred:**

```rust
let current_employee = get_current_employee().await?;
let payroll_period = get_current_payroll_period().await?;
let active_employees = get_active_employees().await?;
```

Avoid vague names:

```rust
let data = ...;
let result = ...;
let x = ...;
```

unless the context is obvious.

### 10. General Principle

Before writing a function call, ask:

> "Has this value already been retrieved or calculated in the current scope?"

If yes, reuse the existing value when it is still valid.

**Primary principle:**

> **Call once when appropriate, store the result, and reuse it. Do not repeatedly execute the same operation without a clear reason.**
