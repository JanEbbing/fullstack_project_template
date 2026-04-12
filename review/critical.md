# Critical Findings

---

## C1 — Blocking database call in async context

**File:** `backend/src/routes/auth.rs` (around `create_token_pair`)

`create_token_pair` is a sync function that acquires the `Mutex<Connection>` lock directly while called from async handlers. It doesn't use `spawn_blocking`, so it blocks an async runtime thread for the duration of the lock + DB write.

Under load, this serialises all token-pair creation through one thread, can cause latency spikes, and in the worst case (slow disk) will exhaust the Tokio thread pool.

The comment in the code acknowledges this but doesn't fix it.

**Fix:** Wrap the body of `create_token_pair` in `tokio::task::spawn_blocking`, or restructure so callers await a `spawn_blocking` future before calling any DB methods.

---

## C2 — CORS allows any origin

**File:** `backend/src/routes/mod.rs`

```rust
let cors = CorsLayer::new()
    .allow_origin(Any)
    .allow_methods(Any)
    .allow_headers(Any);
```

Any website can make credentialed cross-origin requests to the API. Combined with the access token in the `Authorization` header (which the browser won't attach automatically), this is less severe for bearer-token APIs, but it still means no origin isolation at all and makes future missteps more dangerous.

**Fix:** Restrict to the frontend origin from config:

```rust
let origin = state.config.frontend_url
    .parse::<HeaderValue>()
    .expect("Invalid FRONTEND_URL");

let cors = CorsLayer::new()
    .allow_origin(origin)
    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
    .allow_headers([AUTHORIZATION, CONTENT_TYPE])
    .allow_credentials(false);
```

Add `FRONTEND_URL` to `.env.example` and `Config`.

---

## C3 — Weak JWT secret default is not validated

**File:** `.env.example`, `backend/src/config.rs`

The example secret is `dev-secret-change-in-production`. There is no runtime check that prevents this from being used in production. Anyone who knows the value can forge JWTs.

**Fix:** In `config.rs`, fail fast if the secret is too short or matches the known placeholder:

```rust
if config.jwt_secret.len() < 32 {
    panic!("JWT_SECRET must be at least 32 characters");
}
if config.jwt_secret == "dev-secret-change-in-production" {
    panic!("JWT_SECRET must be changed from the default");
}
```

---

## C4 — No rate limiting on auth endpoints

**File:** `backend/src/routes/auth.rs`

`/auth/register`, `/auth/login`, and `/auth/forgot-password` have no rate limiting. An attacker can:
- Brute-force passwords at network speed
- Enumerate valid email addresses via timing or response differences
- Flood users with password-reset emails (spam / account lockout)

**Fix:** Add `tower_governor` or a lightweight in-memory rate limiter as a layer on the auth router. At minimum, limit by IP. Example with `tower_governor`:

```rust
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};

let governor_config = GovernorConfigBuilder::default()
    .per_second(2)
    .burst_size(5)
    .finish()
    .unwrap();

auth_router.layer(GovernorLayer { config: Arc::new(governor_config) })
```

---

## C5 — No request body size limit

**File:** `backend/src/routes/mod.rs`, `backend/src/routes/user.rs`

Axum has no default body size limit. The `content` field in `CreateUserDataRequest` has no `#[validate]` length constraint. A client can POST hundreds of megabytes, causing OOM or disk exhaustion.

**Fix — two parts:**

1. Add a global body limit in the router:

```rust
use axum::extract::DefaultBodyLimit;
app.layer(DefaultBodyLimit::max(1 * 1024 * 1024)) // 1 MB
```

2. Add a validator on `content`:

```rust
#[validate(length(max = 65536))]
pub content: String,
```
