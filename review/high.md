# High Severity Findings

---

## H1 — Internal error details leak to clients

**File:** `backend/src/routes/auth.rs` (multiple locations)

Raw error messages from rusqlite, jsonwebtoken, and other libraries are forwarded to HTTP responses via `AppError::Internal(e.to_string())`. These can reveal:
- Database schema details (column names, constraint names)
- Library versions
- File paths

**Fix:** Log full error internally; return a generic string to clients:

```rust
.map_err(|e| {
    tracing::error!("token creation failed: {}", e);
    AppError::Internal("Internal error".to_string())
})
```

---

## H2 — Refresh token hash comparison is not constant-time

**File:** `backend/src/routes/auth.rs` (refresh handler)

The stored token hash is compared with `!=` (a standard string comparison), which short-circuits on the first differing byte. A remote timing oracle on the response latency could allow an attacker to brute-force valid token hashes one byte at a time.

**Fix:** Use the `subtle` crate:

```rust
use subtle::ConstantTimeEq;

if stored_hash.as_bytes().ct_eq(th.as_bytes()).unwrap_u8() != 1 {
    return Err(AppError::Unauthorized("Invalid refresh token".to_string()));
}
```

The same applies to password-reset token hash comparison.

---

## H3 — Refresh token stored in `localStorage` (XSS-accessible)

**File:** `frontend/src/lib/stores/auth.svelte.ts`

```typescript
localStorage.setItem('refresh_token', refreshToken);
```

Any injected script (XSS, malicious dependency) can read `localStorage` and exfiltrate the long-lived refresh token (7-day expiry), allowing full account takeover.

**Fix:** Serve refresh tokens as `HttpOnly; Secure; SameSite=Strict` cookies. The backend sets the cookie on login/refresh and the frontend never sees the raw token value. The refresh endpoint reads the cookie rather than a JSON body field.

This is a significant design change but is the correct approach for long-lived credentials.

---

## H4 — Logout only revokes refresh token; access token remains valid

**File:** `backend/src/routes/auth.rs` (`logout` handler)

After logout, the access token (15-minute TTL) remains valid. If a token was stolen, the victim logging out provides no protection for 15 minutes.

**Fix:** Add a `token_blacklist` table (or an in-memory TTL set) keyed on the JTI claim of the access token. The `AuthUser` extractor checks the blacklist on every request. Clean up expired entries on startup or via a background task.

---

## H5 — No CSRF protection

**File:** `frontend/src/**`, `backend/src/routes/mod.rs`

State-changing POST endpoints accept JSON from any origin (because CORS is open — see C2). Once CORS is restricted, CSRF becomes less of an issue for `Authorization`-header–based auth, but the current open CORS configuration means cross-site form submissions work freely.

If the fix for C2 (httpOnly cookies) is implemented, CSRF tokens become mandatory.

**Fix:** Either:
1. Keep bearer tokens (don't use cookies) and restrict CORS — this naturally prevents CSRF.
2. If using cookies, add a `SameSite=Strict` attribute and/or a CSRF token header check.

---

## H6 — Missing security headers

**File:** `backend/src/routes/mod.rs`

No Content-Security-Policy, X-Frame-Options, X-Content-Type-Options, or Referrer-Policy headers are set. This leaves the frontend vulnerable to:
- Clickjacking (no `X-Frame-Options`)
- MIME-sniffing (no `X-Content-Type-Options`)
- XSS via inline scripts (no CSP)

**Fix:** Add a `SetResponseHeader` layer or a custom middleware:

```rust
use tower_http::set_header::SetResponseHeaderLayer;

.layer(SetResponseHeaderLayer::if_not_present(
    HeaderName::from_static("x-frame-options"),
    HeaderValue::from_static("DENY"),
))
.layer(SetResponseHeaderLayer::if_not_present(
    HeaderName::from_static("x-content-type-options"),
    HeaderValue::from_static("nosniff"),
))
```

---

## H7 — Password-reset URL not URL-encoded in email

**File:** `backend/src/email/mod.rs`

```rust
let reset_url = format!("{}/reset-password?token={}", self.frontend_url, token);
```

The token is a UUID (hex chars + dashes) so it won't contain problematic characters in practice, but `frontend_url` from config could end with a slash or contain characters that break the URL. More importantly, the URL is inserted raw into an HTML `href` attribute in `templates.rs` without HTML-attribute escaping, which can break the link if the URL contains `"` or `&`.

**Fix:**
1. URL-encode the token parameter.
2. HTML-escape the URL before inserting into the `href` attribute:

```rust
let encoded_token = urlencoding::encode(&token);
let reset_url = format!("{}/reset-password?token={}", frontend_url.trim_end_matches('/'), encoded_token);
// In template: use html_escape::encode_double_quoted_attribute(&reset_url)
```

---

## H8 — No audit logging for security events

**File:** All auth route handlers

Authentication events (login success/failure, registration, password reset, logout, token refresh) produce no structured log entries beyond Axum's request trace. There is no way to detect brute-force attacks, account takeovers, or unusual activity patterns after the fact.

**Fix:** Add structured log lines at `INFO` level for all auth events:

```rust
tracing::info!(
    event = "user_login",
    user_id = %user.id,
    email = %body.email,
    success = true,
);
```

Use `tracing::warn!` for failed attempts. This enables downstream SIEM/alerting without adding a dependency.
