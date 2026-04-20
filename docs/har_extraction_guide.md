# HAR File Extraction Guide

## Overview

Three HAR (HTTP Archive) files contain captured API sessions with real credentials and endpoints:

| File | Size | Platform | Contains |
|------|------|----------|----------|
| `www.dlsite.com_Archive [26-04-20 22-21-11].har` | 108KB | DLSite | 4 requests (purchase/keyword API) |
| `play.dlsite.com_Archive [26-04-20 22-23-38].har` | 784KB | DLSite Play | DLSite play/streaming API |
| `dlsoft.dmm.co.jp_Archive [26-04-20 22-25-45].har` | 329KB | FANZA/DMM | 22 requests (game/product API) |

---

## How to Extract Credentials

### Python Script to Parse HAR Files

```python
import json
import sys

def extract_har_data(filename):
    """Extract API endpoints and cookies from HAR file"""
    with open(filename, 'r', encoding='utf-8') as f:
        har = json.load(f)
    
    entries = har.get('log', {}).get('entries', [])
    print(f"\n📊 {filename}")
    print(f"Total requests: {len(entries)}\n")
    
    for i, entry in enumerate(entries[:10], 1):  # Show first 10
        req = entry.get('request', {})
        url = req.get('url', '')
        method = req.get('method', 'GET')
        
        print(f"{i}. {method} {url[:80]}")
        
        # Extract cookies
        cookies = req.get('cookies', [])
        if cookies:
            print(f"   Cookies ({len(cookies)}):")
            for cookie in cookies[:3]:
                name = cookie.get('name', '?')
                value = cookie.get('value', '')[:40]
                print(f"     - {name}: {value}...")
        
        # Extract headers (useful for auth)
        headers = req.get('headers', [])
        auth_headers = [h for h in headers if 'auth' in h.get('name', '').lower()]
        if auth_headers:
            print(f"   Auth headers:")
            for h in auth_headers:
                print(f"     - {h.get('name')}: {h.get('value')[:40]}...")
        
        print()

# Usage
extract_har_data('www.dlsite.com_Archive [26-04-20 22-21-11].har')
extract_har_data('play.dlsite.com_Archive [26-04-20 22-23-38].har')
extract_har_data('dlsoft.dmm.co.jp_Archive [26-04-20 22-25-45].har')
```

### Run Extraction

```bash
cd /Users/phillui/Documents/self/personal_inventory

# Python extraction
python3 << 'EOF'
import json

for filename in [
    'www.dlsite.com_Archive [26-04-20 22-21-11].har',
    'play.dlsite.com_Archive [26-04-20 22-23-38].har',
    'dlsoft.dmm.co.jp_Archive [26-04-20 22-25-45].har'
]:
    try:
        with open(filename, 'r', encoding='utf-8') as f:
            har = json.load(f)
        entries = har.get('log', {}).get('entries', [])
        
        print(f"\n{'='*60}")
        print(f"📄 {filename}")
        print(f"{'='*60}")
        print(f"Total requests: {len(entries)}")
        
        for i, entry in enumerate(entries[:5], 1):
            url = entry.get('request', {}).get('url', '')
            print(f"\n  {i}. {url}")
    except Exception as e:
        print(f"Error reading {filename}: {e}")
EOF
```

---

## Key Data Points

### Steam
```
API Key: C1AE1AEA9578207CC02DCBC27FCA6E0E
Domain: https://kgy-production.xyz
Status: ✅ Ready to use
```

### DLSite (from www.dlsite.com HAR)
```
Base URL: https://www.dlsite.com/home/api
Endpoints:
  - /recruit/info/api?locale=ja_JP
  - /=/popularKeyword.json?limit=10&locale=ja-jp

Key Cookies:
  - __cf_bm (Cloudflare Bot Management)
  - locale (ja-jp)
  - _trust360_ (Trust360 fraud detection)
```

### DLSite Play (from play.dlsite.com HAR)
```
Base URL: https://play.dlsite.com
Status: Contains streaming/play API calls (784KB of data)
Extract: Use jq to parse individual requests
```

### FANZA/DMM (from dlsoft.dmm.co.jp HAR)
```
API Bases:
  - https://api.cds.dmm.co.jp/v1 (games/delivers)
  - https://support.dmm.co.jp/api (metadata)

Key Endpoints:
  - /v1/delivers/list/statuses?deliver_ids=...
  - /api/announcement-bars?bar_place_id=168

Key Cookies:
  - top_pv_uid (session ID)
  - top_dummy (test cookie)
  - i3_ab (A/B testing)
```

---

## Integration with Phase 9

### Step 1: Extract Real API Responses
```bash
# Parse HAR files and save API responses as fixtures
python3 << 'EOF'
import json

for har_file in ['www.dlsite.com_Archive [26-04-20 22-21-11].har']:
    with open(har_file, 'r', encoding='utf-8') as f:
        har = json.load(f)
    
    entries = har['log']['entries']
    for entry in entries:
        req = entry['request']
        resp = entry['response']
        url = req['url']
        
        # Save response body as fixture
        if resp.get('content', {}).get('text'):
            filename = url.split('/')[-1].split('?')[0]
            with open(f'backend/plugins/tests/fixtures/dlsite_{filename}.json', 'w') as out:
                out.write(resp['content']['text'])
EOF
```

### Step 2: Update Fixture Loaders
Modify `backend/plugins/src/ecosystem/dlsite_fixture.rs` to use real API response structure from HAR files instead of synthetic fixtures.

### Step 3: Add Session Cookie Handling
```rust
// In SyncService
pub struct DlsiteSession {
    cookies: HashMap<String, String>,
    api_base: String,
}

impl DlsiteSession {
    pub fn from_har_file(path: &str) -> Result<Self, SyncError> {
        let har = parse_har(path)?;
        let cookies = extract_cookies_from_har(&har)?;
        Ok(DlsiteSession {
            cookies,
            api_base: "https://www.dlsite.com/home/api".to_string(),
        })
    }
}
```

### Step 4: Real API Integration
Once Phase 9 is ready, use HAR data to:
1. Get real API endpoints
2. Extract session cookies
3. Build authenticated HTTP requests
4. Parse real response structures

---

## Security Notes

### DO NOT
- ❌ Commit HAR files to Git (contains credentials)
- ❌ Share HAR files publicly (contains session data)
- ❌ Use HAR files in CI/CD (session cookies expire)

### DO
- ✅ Use HAR files locally for development
- ✅ Extract API endpoints and use for fixture structure
- ✅ Re-capture HAR files periodically (cookies expire)
- ✅ Store real credentials in `.env.local` or GitHub Secrets

---

## How to Re-Capture HAR Files

### Chrome DevTools
1. Open **DevTools** (F12)
2. Go to **Network** tab
3. Check "Preserve log"
4. Navigate to the website/API
5. Right-click → **Save all as HAR with content**
6. Save file with timestamp: `www.dlsite.com_Archive [YYYY-MM-DD HH-MM-SS].har`

### Network Sniffing Tools
- **Charles Proxy**: Enterprise HTTP proxy, easy HAR export
- **Fiddler**: Free HTTP debugging tool
- **mitmproxy**: Open-source proxy with HAR support

---

## Next Steps

1. **Parse HAR files** using provided Python scripts
2. **Extract real API endpoints** and response structures
3. **Create real fixture files** from HAR responses
4. **Update fixture loaders** to use real API structure
5. **Implement Phase 9** with real API calls using HAR session data

---

## Reference

- HAR Format: https://www.softwareishard.com/blog/har-12-spec/
- jq tool for JSON parsing: https://stedolan.github.io/jq/
- Python json module: https://docs.python.org/3/library/json.html

**Last Updated**: 2026-04-20  
**Status**: Ready for Phase 9 real API integration
