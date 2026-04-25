---
name: web-fetch
description: Fetch web pages and extract readable content

structures:
  - name: fetch
    path: ../structures/web-fetch
    summary: "Fetch web page content"
    scenarios: ["fetch URL", "web content extraction"]
    weight: 1.0

config:
  default_timeout: 15
---

# Web Fetch Skill

Fetch web page content and extract readable text.

## Usage

```bash
cogtome run web-fetch --input '{"url": "https://example.com"}'
```

## Output

Returns the page URL and extracted content.
