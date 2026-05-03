# Sprint 53 route collision prompt — SPA reload `/browse` / `/curators`

**Date** : 2026-05-03  
**Statut** : investigation/pivot candidate, not a PASS review.  
**Contexte commit** : `190b582` opened Sprint 53 as P2P smoke test
multi-platform + VPS bootstrap. Current dirty worktree already contains
frontend/daemon changes; verify before editing.

---

## Placement planning recommande

This issue should be tracked as an **S53 Phase A pivot candidate** if Browse or
Curators are needed during the LAN smoke test. It is a daemon-served shell bug
found while validating the runtime surface, so it can fit Phase A only if the
phase review explicitly records the scope deviation.

If Phase A continues as "P2P logs only" and the UI is not needed, keep the fix
out of Phase A and carry it into `sprint54_audit_plan.md` as a P2/P1 frontend
shell reliability item.

Do not hide it as a token-race fix. The reproduced failure is a **route
collision**: browser document navigation to `/browse` or `/curators` cannot
carry `x-sbfb-token` because React has not started yet.

Required planning updates when implemented:

- `.planning/active/sprint53_phase_A_preflight.md` : add a pivot/deviation note
  if fixed during Phase A.
- `.planning/active/sprint53_phase_A_review.md` : include `## Verdict: PASS`
  only after reload tests prove `/browse` and `/curators` serve the SPA document.
- `.planning/active/sprint53_verification.md` : add fail-fast rows for direct
  reload on `/browse` and `/curators`, plus API namespace checks.
- `.planning/active/sprint54_audit_plan.md` : carry any compatibility or docs
  follow-up that remains open.
- `docs/shell/PATTERNS.md` and `docs/architecture/LAUNCHER.md` : document the
  rule that daemon JSON routes must not occupy browser SPA route paths.

Suggested commit shape if fixed in Sprint 53:

```text
fix(sprint53): Sprint 53 Phase A — fix daemon-served SPA reload route collision
```

The commit body must say whether this is a Phase A pivot and list browser
reload evidence, Rust tests, frontend tests, and docs updates.

---

## Prompt to verify and fix

You are working in `C:\Users\FlowUP\Documents\Code\nexus`.

Goal: verify and fix the daemon-served React shell reload failure where F5 on
`http://127.0.0.1:7654/browse` or `/curators` returns `401 missing or invalid
token`.

### Ground truth to preserve

- The daemon serves the Vite build via `--web-root ./web/dist`.
- `GET /auth/token` is public only after Host + Origin loopback checks.
- Browser API calls must use `authFetch()` and send `x-sbfb-token`.
- Browser document navigation cannot include JS-injected headers.
- Preserve loopback auth invariants; do not make `/browse` or `/curators`
  publicly return JSON.

### Required investigation

1. Read these files first:
   - `crates/nexus-shell-daemon/src/http.rs`
   - `crates/nexus-shell-daemon-core/src/auth.rs`
   - `web/src/App.tsx`
   - `web/src/api/auth.ts`
   - `web/src/api/daemon.ts`
   - `web/tests/browse-daemon-offline.spec.ts`
   - `web/tests/curators-flow.spec.ts`
   - `docs/shell/PATTERNS.md`
   - `docs/architecture/LAUNCHER.md`
2. Confirm the route collision:
   - UI routes: `web/src/App.tsx` has `/browse` and `/curators`.
   - daemon API routes: `http.rs` has authenticated `GET /browse` and
     `GET /curators`.
   - fallback SPA service is registered after daemon routes.
   - `auth_required` returns `401 missing or invalid token` before React can
     fetch `/auth/token`.
3. Reproduce with a browser/network capture, not only PowerShell:
   - direct navigation/reload to `/browse` sends no `x-sbfb-token`;
   - in-app navigation from `/` first gets `/auth/token`, then API `/browse`
     carries `x-sbfb-token` and succeeds.

### Preferred fix direction

Move daemon JSON API routes out of SPA route paths. Prefer a namespaced API such
as:

```text
/api/daemon/info
/api/daemon/curators
/api/daemon/curators/subscribe
/api/daemon/curators/{pubkey}
/api/daemon/browse
/api/daemon/diagnostic/neighborhood
/api/daemon/panic/wipe
```

Then update `web/src/api/daemon.ts` to call the new paths. Remove or gate the
old bare JSON routes so document reloads for `/browse` and `/curators` reach the
SPA fallback instead of the auth middleware.

If you choose an alternative, document why it is safer than API namespacing.
Reject fixes that only wait for `fetchAuthToken()` before `createRoot()`: that
does not affect the browser's initial document request on F5.

### Tests and proof required

Add or update tests so the regression cannot return:

- Rust HTTP/router test: with `web_root` enabled, document-style `GET /browse`
  and `GET /curators` without token return `200 text/html` containing the SPA
  root, not `401`.
- Rust HTTP/router test: JSON API calls on the new namespace still require and
  accept `x-sbfb-token`.
- Frontend unit tests: `listBrowse()` and `listCurators()` call the new
  namespaced paths.
- Playwright/manual browser evidence:
  - direct `page.goto("/browse")` then reload does not show
    `missing or invalid token`;
  - direct `page.goto("/curators")` then reload does not show
    `missing or invalid token`;
  - in-app navigation still calls the API with token.

Useful one-off Playwright capture:

```bash
cd web
node - <<'JS'
const { chromium } = require('playwright');
(async () => {
  const browser = await chromium.launch({ headless: true });
  const page = await browser.newPage();
  page.on('request', req => {
    const url = req.url();
    if (/\/(auth\/token|health|browse|curators|api\/daemon)(\?|$)/.test(url)) {
      const h = req.headers();
      console.log('REQ', req.method(), url, 'accept=', h.accept, 'token=', !!h['x-sbfb-token']);
    }
  });
  page.on('response', async res => {
    const url = res.url();
    if (/\/(browse|curators|api\/daemon)(\?|$)/.test(url)) {
      console.log('RES', res.status(), url, res.headers()['content-type']);
    }
  });
  await page.goto('http://127.0.0.1:7654/browse', { waitUntil: 'domcontentloaded' });
  await page.reload({ waitUntil: 'domcontentloaded' });
  await page.goto('http://127.0.0.1:7654/curators', { waitUntil: 'domcontentloaded' });
  await page.reload({ waitUntil: 'domcontentloaded' });
  await browser.close();
})();
JS
```

### Verification commands

Run the smallest relevant suite first, then broaden before commit:

```bash
cargo fmt --all --check
cargo test -p nexus-shell-daemon --locked http
cd web && npm run test:unit -- daemon auth
cd web && npm run build
```

Before a phase commit, follow the repo process:

```bash
python scripts/agent/agentctl.py prompt --kind preflight --sprint 53 --phase A --depth deep
python scripts/agent/agentctl.py precommit-lightcheck
python scripts/agent/agentctl.py auditor-gate --message-file .git/COMMIT_EDITMSG
```

### Review criteria

Return PASS only if all of these are true:

- F5 on `/browse` and `/curators` serves the SPA document.
- API JSON calls still require `x-sbfb-token`.
- `GET /auth/token` remains public only under loopback Host + Origin checks.
- The fix is documented in planning and shell/launcher docs.
- The commit body explicitly separates "token bootstrap race" from "route
  collision on document navigation".
