---
name: browser-fetch
description: Fetch web pages with JavaScript rendering support. Uses headless browser (camoufox) for JS-heavy pages, with jina.ai reader as fallback. Use when you need to extract content from pages that require JavaScript rendering (WeChat articles, SPAs, etc.).

structures:
  - name: browser-fetch
    path: ../structures/browser-fetch
    summary: "Fetch web page with JS rendering"
    scenarios: ["fetch URL", "JS-heavy page", "WeChat article"]
---

# Browser Fetch

Fetch web pages with full JavaScript rendering support.

## Input Schema

```json
{
  "url": "string (required) - URL to fetch"
}
```

## Output Schema

```json
{
  "url": "string - Original URL",
  "content": "string - Extracted text content"
}
```

## Examples

```
cogtome run browser-fetch --input '{"url": "https://example.com"}'
cogtome run browser-fetch --input '{"url": "https://mp.weixin.qq.com/s/..."}'
```

## Architecture

- **camoufox-fetch**: Primary unit using headless browser
- **jina-reader**: Fallback using Jina AI reader API
