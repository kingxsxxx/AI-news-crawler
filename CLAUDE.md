# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is an AI news aggregator desktop application built with Tauri v2. It fetches AI-related news from multiple RSS feeds and web sources, stores them locally in SQLite, and provides a React-based web UI for reading and searching. The app features AI-powered Chinese summarization (configurable via environment variables) and full-text search with SQLite FTS5.

## Common Commands

```bash
npm install                    # Install frontend dependencies
npm run dev                    # Run Vite dev server (frontend only)
npm run build                  # TypeScript check + Vite build (produces dist/)
npm run tauri:dev              # Run full Tauri app in development mode
npm run tauri:build            # Build production Tauri application
cargo check --manifest-path src-tauri/Cargo.toml  # Validate Rust compilation
```

## Architecture

### Tech Stack
- **Desktop Framework**: Tauri v2 (Rust backend + Webview frontend)
- **Frontend**: React 18.2 + TypeScript, Vite 5.2, Zustand 4.4 (state)
- **Backend**: Rust 2021, SQLite (rusqlite with bundled lib)
- **Key Crates**: reqwest (HTTP), scraper (HTML parsing), rss (RSS feeds), chrono (time)

### Project Structure
- `src/` - React frontend (main.tsx, App.tsx, lib/api.ts, types/, components/)
- `src-tauri/src/` - Rust backend (main.rs entry, lib.rs with all commands/db/crawler)
- Database stored in OS app data directory as `news.db` (auto-created on startup)

### Tauri Commands (Backend API)
Defined in `src-tauri/src/lib.rs`, called via `invoke()` from `src/lib/api.ts`:
- `health` - Health check endpoint
- `articles_list` - Paginated article listing with optional category filter
- `article_get` - Single article by ID
- `article_bookmark` / `article_mark_read` - Toggle article state
- `search_query` - FTS5 full-text search with bm25 ranking
- `manual_add` - Add article from URL (fetches and parses page)
- `crawler_run_once` - Fetch from all active sources (up to 20, processes all source types)
- `articles_regenerate_summaries` - Batch regenerate AI summaries for template-based articles
- `settings_get` / `settings_update` - User preferences
- `ai_summarize` - Generate AI summary for content
- `open_external` - Open URL in system browser

### Database Schema
- `articles` - id, title, summary, content, url (unique), source, category, published_at, fetched_at, heat_score, is_read, is_bookmarked, image_url
- `articles_fts` - FTS5 virtual table (title, summary, content) with unicode61 tokenizer
- `settings` - theme, ai_model, ai_base_url, ai_api_key, ai_summary_enabled
- `sources` - name (unique), url, source_type, is_active

### News Source Types
- **RSS** - Fetches feed, extracts items (title, link, description, enclosure image), up to 12 items per source
- **WEB** - HTML scraping (currently configured for Anthropic news page), parses anchor tags
- **API** - JSON API response parsing (expects `{data: [{title, url, published_at}]}` format)
- **GITHUB_TRENDING** - Scrapes GitHub trending pages, extracts repo info (stars, language, description, og:image)

### Key Patterns
- **URL Deduplication**: URLs normalized (trim, lowercase, trailing slash removed) before storage
- **Image Fallback**: picsum.photos with deterministic seed based on source/title keywords (openai, anthropic, google, meta, microsoft, xai)
- **Chinese Summarization**: AI via OpenAI-compatible API (DashScope/Qwen default), falls back to `make_zh_brief()` template
- **Search**: FTS5 prefix matching (`token*`), bm25 ranking, results limited to 100

### Default News Sources (seeded on first run)
**International - AI/Tech:**
OpenAI Blog, Google AI Blog, DeepMind Blog, Anthropic News, MIT Tech Review AI, VentureBeat AI, Hacker News Frontpage, Hacker News AI

**Chinese - AI/Tech:**
雷锋网 AI, 钛媒体, 36氪, 机器之心, 量子位, 智东西, InfoQ中文

**GitHub:**
GitHub Trending (all), GitHub Trending AI, GitHub Trending Python, GitHub Trending TypeScript, GitHub Trending Rust, HelloGitHub月刊

## Environment Variables

Required for AI summarization (see `.env.example`):
- `AI_BASE_URL` - Base URL for AI API (e.g., "https://dashscope.aliyuncs.com/compatible-mode/v1")
- `AI_API_KEY` - API key for the AI service
- `AI_MODEL` - Model name (default: "qwen3-max")

Env files loaded in order: `.env`, `.env.local`, `../.env`, `../.env.local` (dotenvy)

## Code Style
- TypeScript with strict mode
- React components: `PascalCase` (e.g., `ArticleCard.tsx`)
- Rust: `snake_case` for functions/modules, `PascalCase` for types
- Indentation: 2 spaces for TS/JSON/CSS, 4 spaces for Rust

## Development Notes

- Backend is monolithic in `lib.rs` (not yet modularized into commands/, db/, crawler/)
- Use `cargo check` before pushing Rust changes to catch compile errors
- AI summarization gracefully degrades if API keys not configured
- Article content truncated to ~1200 chars for storage efficiency
- Crawler processes up to 20 sources per run (LIMIT 20 in SQL)
- HTTP client defaults to 127.0.0.1:7897 proxy if no HTTP_PROXY env var is set
- OG image fetching and AI summarization during crawl are disabled (commented out) to avoid timeouts
- AI summaries use exponential backoff retry (3 attempts, 2/4/8 second delays) with 1-second rate limiting between calls
- Date normalization: various formats (RFC3339, RFC2822, etc.) are normalized to ISO 8601 for proper sorting
- Template summaries ("这篇英文资讯围绕...") are used as fallback when AI is unavailable; can be regenerated via `articles_regenerate_summaries`