# Code Review — Full-Stack Project Template

**Date:** 2026-03-30
**Scope:** Backend (Rust/Axum), Frontend (SvelteKit), Docker, CI/CD

## Files Reviewed

- `backend/src/**` (all source files)
- `backend/tests/**`
- `frontend/src/**`
- `frontend/tests/**`
- `Dockerfile`, `docker-compose.yml`, `.dockerignore`
- `.github/workflows/ci.yml`
- `.pre-commit-config.yaml`
- `scripts/**`
- `.env.example`

## Summary

| Severity | Count | File |
|----------|-------|------|
| Critical | 5 | [critical.md](critical.md) |
| High | 8 | [high.md](high.md) |
| Medium | 9 | [medium.md](medium.md) |
| Low / Nit | 12 | [low.md](low.md) |
| **Total** | **34** | |

## Recommended Priorities

**Phase 1 — Before any real usage:**
1. Restrict CORS origins (C2)
2. Add request body size limits (C5)
3. Add rate limiting to auth endpoints (H4)
4. Fix blocking DB call in `create_token_pair` (C1)
5. Add `DefaultBodyLimit` middleware (C5 / L5)

**Phase 2 — Before production:**
6. Add security headers — CSP, X-Frame-Options, etc. (H7)
7. Switch refresh tokens from `localStorage` to `httpOnly` cookies (H3)
8. Implement access token blacklist for logout (H6)
9. Add audit logging for auth events (M3)
10. Add explicit DB expiry check in refresh handler (M7)

**Phase 3 — Polish:**
11. Add API versioning (M8)
12. Add database connection pooling (M1)
13. Add health-check DB probe (L3)
14. Set non-root user in Dockerfile (L4)
15. Add Dependabot config (L12)
