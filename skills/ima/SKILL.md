---
name: ima
description: IMA知识库 — 腾讯ima.copilot知识库管理。支持微信公众号文章、网页、PDF、Word等上传到知识库，以及知识库内容搜索和查看。使用前需设置 IMA_CLIENT_ID 和 IMA_API_KEY 环境变量（或 ~/.config/ima/ 目录配置）。
structures:
  - name: ima-list-kb
    path: ../structures/ima-list-kb
    summary: "列出用户的所有知识库"
    scenarios: ["查看知识库列表", "搜索知识库"]
  - name: ima-add-url
    path: ../structures/ima-add-url
    summary: "添加网页/微信文章到知识库"
    scenarios: ["添加微信文章", "添加网页到知识库"]
  - name: ima-get-article
    path: ../structures/ima-get-article
    summary: "通过IMA中转获取微信文章内容"
    scenarios: ["抓取微信文章", "绕过微信验证墙"]
---

# IMA 知识库

IMA 是腾讯推出的 AI 知识库工作台，支持微信公众号文章、网页、文档等上传和管理。

## 前置要求

**环境变量配置**（二选一）：

```bash
# 方式1: 环境变量
export IMA_CLIENT_ID="your_client_id"
export IMA_API_KEY="your_api_key"

# 方式2: 文件配置
mkdir -p ~/.config/ima
echo -n "your_client_id" > ~/.config/ima/client_id
echo -n "your_api_key" > ~/.config/ima/api_key
```

API Key 获取地址：https://ima.qq.com/agent-interface

## Units

- `ima-list-knowledge-bases` — 列出用户的所有知识库
- `ima-add-url` — 添加网页/微信公众号文章到知识库
- `ima-search-knowledge` — 在知识库中搜索内容
- `ima-get-media-info` — 获取知识库条目的详细信息
- `ima-get-knowledge-base` — 获取知识库详情

## Structures

### ima-get-article — 微信文章中转获取

通过 IMA 知识库中转，绕过微信验证墙抓取公众号文章。

```bash
cogtome structure run ima-get-article --input '{
  "urls": ["https://mp.weixin.qq.com/s/xxxxx"],
  "knowledge_base_id": "<kb_id>",
  "search_query": "mp.weixin.qq.com"
}'
```

**原理**：IMA 是腾讯内部产品，请求微信文章时走内部信任通道，可绕过第三方验证墙。

### ima-add-url — 添加微信文章到知识库

```bash
cogtome structure run ima-add-url --input '{
  "urls": ["https://mp.weixin.qq.com/s/xxxxx"],
  "knowledge_base_id": "<kb_id>"
}'
```

### ima-list-kb — 列出知识库

```bash
cogtome run ima --input '{"action": "list"}'
```
