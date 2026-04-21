# HAR Extraction Guide

Use this when connector work needs fresh real-world request shape data. Keep it local. Keep it sanitized. Do not turn this repo into a credential dump.

## Local capture files

Current local captures at repo root:

| File | Platform | What it is useful for |
|---|---|---|
| `www.dlsite.com_Archive [26-04-20 22-21-11].har` | DLSite | Library/auth request shape and cookie names |
| `play.dlsite.com_Archive [26-04-20 22-23-38].har` | DLSite Play | Streaming/play request shape |
| `dlsoft.dmm.co.jp_Archive [26-04-20 22-25-45].har` | FANZA/DMM | Library/auth request shape and cookie names |

Do **not** commit new HAR files. They stay local.

## What to extract

For each platform, record only the minimum operator knowledge the connector needs:

1. Final library endpoint URL path
2. Required header **names** and cookie **names**
3. Login/session validation selectors
4. Response shape notes needed for fixtures/tests
5. Any throttling, redirect, or OTP checkpoints

Do **not** keep live values for cookies, auth headers, API keys, emails, account IDs, OTP codes, or raw purchased content payloads.

## Sanitization rules

Before any captured data influences code or docs:

- Strip cookie values. Keep cookie names only.
- Strip auth header values. Keep header names only.
- Replace account-specific IDs with placeholders unless the shape itself matters.
- Replace timestamps with examples when exact values are irrelevant.
- Never paste raw HAR snippets into committed docs.
- Never use HAR files in CI.

If you need a fixture, convert the response into a sanitized minimal-structure JSON fixture first.

## Safe local inspection commands

List request methods, URLs, header names, and cookie names without printing secrets:

```bash
python3 <<'PY'
import json
from pathlib import Path

for path in [
    Path("www.dlsite.com_Archive [26-04-20 22-21-11].har"),
    Path("play.dlsite.com_Archive [26-04-20 22-23-38].har"),
    Path("dlsoft.dmm.co.jp_Archive [26-04-20 22-25-45].har"),
]:
    if not path.exists():
        continue
    har = json.loads(path.read_text(encoding="utf-8"))
    print(f"\n## {path.name}")
    for entry in har.get("log", {}).get("entries", [])[:20]:
        request = entry.get("request", {})
        headers = sorted({header.get("name", "") for header in request.get("headers", []) if header.get("name")})
        cookies = sorted({cookie.get("name", "") for cookie in request.get("cookies", []) if cookie.get("name")})
        print(request.get("method", "GET"), request.get("url", ""))
        print("  headers:", ", ".join(headers) or "(none)")
        print("  cookies:", ", ".join(cookies) or "(none)")
PY
```

Search connector code and docs for stale capture debt:

```bash
rg -n "TODO\\(network-inspection\\)|selector|cookie|sanitize|HAR" \
  backend/plugins/src/ecosystem \
  docs/har_extraction_guide.md
```

## Platform notes

### DLSite / FANZA

- Use the local HAR files above to confirm the latest library endpoint path and cookie/header names.
- Capture the selector or URL that proves the session is authenticated.
- Convert only sanitized response structure into fixtures or tests.

### Kindle

- Prefer documenting the verified browser/OTP path or the explicit fallback contract.
- If the capture includes OTP or account-specific identifiers, strip them before writing any note.

### BookWalker

- Capture the verified library endpoint and any selector needed to prove login success.
- Keep only sanitized response structure for fixture work.

## After updating connector assumptions

Run the plugin suite and keep the docs in sync with the code:

```bash
cd backend && cargo test -p plugins
```

If the connector contract changed, update this guide in the same change so later work does not depend on memory.
