# Low / Nit Findings

---

## L1 — Hardcoded static file path construction

**File:** `backend/src/routes/mod.rs`

```rust
let fallback_file = format!("{}/200.html", static_dir);
```

If `static_dir` ends with `/`, the path becomes `path//200.html`. On most platforms this works, but it's fragile.

**Fix:**

```rust
use std::path::Path;
let fallback_file = Path::new(&static_dir).join("200.html");
```

---

## L2 — Inconsistent error handling pattern in auth.rs

**File:** `backend/src/routes/auth.rs`

Some paths use `.map_err(|e| AppError::Internal(format!("...: {e}")))` and others use `?` with `From` conversions. The mixed style makes it harder to add the logging fix from H1 consistently.

**Fix:** Define a small helper or use the existing `AppError` `From<rusqlite::Error>` and `From<jsonwebtoken::errors::Error>` impls where they exist, and log at the conversion site.

---

## L3 — Logout silently swallows server errors in Navbar

**File:** `frontend/src/lib/components/Navbar.svelte`

```typescript
} catch {
    // Proceed with local logout even if server call fails
}
```

The user clears local state but doesn't know the server-side revocation failed. The refresh token is still valid on the server.

**Fix:** Either show a toast notification on error, or at least log with `console.warn`. Proceeding with local logout is the right UX choice; just tell the user.

---

## L4 — Dockerfile runs as root

**File:** `Dockerfile`

The final stage has no `USER` directive. The process runs as root inside the container. A container escape gives the attacker a root shell.

**Fix:**

```dockerfile
RUN useradd --system --uid 1001 --no-create-home appuser
USER 1001
```

Add this before the `CMD` in the final stage.

---

## L5 — No request body size limit (duplicate of C5 infrastructure note)

Already covered in C5. The fix (DefaultBodyLimit middleware) should be the primary location.

---

## L6 — No explicit timestamp format contract

**File:** `backend/src/db/migrations.rs`

Timestamps use `datetime('now')` which produces `YYYY-MM-DD HH:MM:SS` (no timezone, no 'Z' suffix). The Rust code parses them as RFC 3339 in some places. This works for SQLite's default output but is undocumented and fragile if the format ever changes.

**Fix:** Store timestamps as RFC 3339 strings explicitly:

```sql
created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
```

And document the format in a comment in `migrations.rs`.

---

## L7 — Integration tests use `:memory:` SQLite only

**File:** `backend/tests/common/mod.rs`

In-memory SQLite is fast and fine for the current tests, but won't catch issues specific to WAL mode, file locking, or the `create_dir_all` path in `db::init`.

**Fix:** Consider adding a separate CI step that runs the same tests against a real file-backed SQLite database.

---

## L8 — No Dependabot or automated dependency updates

**File:** `.github/workflows/`

No `dependabot.yml` means dependency security advisories won't generate PRs automatically.

**Fix:** Add `.github/dependabot.yml`:

```yaml
version: 2
updates:
  - package-ecosystem: "cargo"
    directory: "/backend"
    schedule:
      interval: "weekly"
  - package-ecosystem: "npm"
    directory: "/frontend"
    schedule:
      interval: "weekly"
  - package-ecosystem: "github-actions"
    directory: "/"
    schedule:
      interval: "weekly"
```

---

## L9 — CI docker-build job only runs on `main`

**File:** `.github/workflows/ci.yml`

The `build-docker` job has an `if: github.ref == 'refs/heads/main'` condition (or equivalent). Dockerfile errors aren't caught on PRs.

**Fix:** Remove the branch restriction from `build-docker`, or add a separate `docker-build-check` job that runs on PRs but doesn't push the image.

---

## L10 — Missing OpenAPI documentation

**File:** No API docs found

No schema or documentation for the API. Frontend and any future integrations must be inferred from source.

**Fix:** Add `utoipa` + `utoipa-swagger-ui` to the backend. Decorate route handlers with `#[utoipa::path(...)]` and expose `/api-docs` in dev mode only.

---

## L11 — No GDPR/cookie consent for `localStorage` tokens

**File:** `frontend/src/lib/stores/auth.svelte.ts`

Storing tokens in `localStorage` is persistent storage and may require user consent under GDPR/ePrivacy in the EU.

**Fix:** If targeting EU users, display a minimal consent notice before setting storage. If switching to `httpOnly` session cookies (see H3), this concern may not apply depending on jurisdiction.

---

## L12 — No non-root user in docker-compose healthcheck

**File:** `docker-compose.yml`

No health check is configured, so Docker Compose doesn't know when the app is ready.

**Fix:**

```yaml
healthcheck:
  test: ["CMD", "curl", "-f", "http://localhost:3000/api/health"]
  interval: 10s
  timeout: 5s
  retries: 3
  start_period: 10s
```
