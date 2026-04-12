# Medium Severity Findings

---

## M1 — Single serialised database connection

**File:** `backend/src/main.rs`, `backend/src/db/mod.rs`

All requests share one `Arc<Mutex<Connection>>`. Every DB operation — including reads — serialises behind a single lock. Under any real concurrency this will be the bottleneck.

SQLite supports WAL mode (already enabled), which allows concurrent readers with one writer, but rusqlite's `Connection` is not `Send`/`Sync` by design, so using multiple connections requires care.

**Fix options (in order of effort):**
1. Use `r2d2` + `r2d2_sqlite` for a small connection pool (3–5 connections).
2. Switch to `sqlx` with an async SQLite pool (`sqlx::SqlitePool`), which eliminates the `spawn_blocking` pattern entirely.
3. At minimum, use separate read connections from write connections.

---

## M2 — Weak default password policy

**File:** `backend/src/routes/auth.rs` (`RegisterRequest`)

```rust
#[validate(length(min = 8, max = 128))]
pub password: String,
```

Length 8 is the only requirement. "12345678" and "password" pass. Given that this template is meant for real projects, some minimum complexity check is appropriate.

**Fix:** Add a custom validator:

```rust
fn validate_password(password: &str) -> Result<(), ValidationError> {
    if !password.chars().any(|c| c.is_ascii_digit())
        || !password.chars().any(|c| c.is_alphabetic())
    {
        return Err(ValidationError::new("password_complexity"));
    }
    Ok(())
}
```

Or use the `zxcvbn` crate for a score-based check.

---

## M3 — No audit trail for auth events (see also H8)

Already covered in H8, but repeated here because the medium-severity aspect is lack of traceability for non-attack scenarios: support tickets, compliance audits, debugging. H8 covers the security dimension; this is the operational dimension.

---

## M4 — Frontend auth guard is UX-only; document this clearly

**File:** `frontend/src/lib/components/AuthGuard.svelte`

`AuthGuard` redirects unauthenticated users to `/login` client-side. A user can bypass it by manually setting `auth.isAuthenticated = true` in the browser console. This is expected and fine — the backend enforces actual auth. But it is not documented anywhere, and a future developer might assume the frontend guard is a security boundary.

**Fix:** Add a comment in `AuthGuard.svelte`:

```svelte
<!--
  UX-only: redirects unauthenticated users to /login.
  Not a security boundary — all protected data is enforced server-side.
-->
```

---

## M5 — `initAuth` has no timeout; loading state can hang indefinitely

**File:** `frontend/src/lib/stores/auth.svelte.ts`

If the backend is slow or unreachable, `initAuth` awaits the fetch indefinitely. The page stays in the loading state forever.

**Fix:**

```typescript
const controller = new AbortController();
const timeoutId = setTimeout(() => controller.abort(), 5000);
try {
    const response = await fetch('/api/auth/refresh', {
        method: 'POST',
        signal: controller.signal,
        // ...
    });
    // ...
} finally {
    clearTimeout(timeoutId);
}
```

---

## M6 — Password-reset token format not validated before DB query

**File:** `backend/src/routes/auth.rs` (`reset_password` handler)

The `token` field in `ResetPasswordRequest` has no format validation. Any string — including empty string — is hashed and queried against the database. This wastes work and, if the validator library is bypassed (e.g., fuzz testing), can cause unexpected behaviour.

**Fix:**

```rust
#[validate(length(equal = 36))]  // UUID canonical form
pub token: String,
```

Or validate that the string only contains hex characters and dashes.

---

## M7 — Refresh handler doesn't check database `expires_at`

**File:** `backend/src/routes/auth.rs` (refresh handler)

The refresh flow:
1. Verify JWT signature and expiry (handled by `jsonwebtoken`)
2. Look up the hashed token in DB
3. Check `revoked = 0`
4. Does **not** check `expires_at` column

If a clock skew or bug means the JWT hasn't expired but the DB record has (or vice versa), the DB `expires_at` is silently ignored. The DB expiry column is dead code.

**Fix:** Add an explicit check:

```rust
let expires_at: String = row.get(3)?;
let expires = DateTime::parse_from_rfc3339(&expires_at)
    .map_err(|_| AppError::Internal("Invalid expiry format".into()))?;
if expires < Utc::now() {
    return Err(AppError::Unauthorized("Token expired".to_string()));
}
```

---

## M8 — No API versioning

**File:** `backend/src/routes/mod.rs`

Routes are nested under `/api/` with no version component. Any breaking change requires a flag day across all clients.

**Fix:** Nest under `/api/v1/` from the start. It costs nothing now and avoids a painful migration later:

```rust
Router::new().nest("/api/v1", api_router)
```

---

## M9 — Health check doesn't probe the database

**File:** `backend/src/routes/health.rs`

```rust
pub async fn health_check() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}
```

A load balancer or container orchestrator will consider the instance healthy even if the database connection is broken or the mutex is poisoned. This masks failures and delays incident response.

**Fix:**

```rust
pub async fn health_check(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let db = Arc::clone(&state.db);
    tokio::task::spawn_blocking(move || {
        db.lock()
            .map_err(|_| AppError::Internal("DB mutex poisoned".into()))?
            .query_row("SELECT 1", [], |_| Ok(()))
            .map_err(|e| AppError::Internal(e.to_string()))
    })
    .await
    .map_err(|_| AppError::Internal("spawn_blocking failed".into()))??;

    Ok(Json(json!({ "status": "ok" })))
}
```
