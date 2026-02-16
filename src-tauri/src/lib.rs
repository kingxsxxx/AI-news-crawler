use std::sync::Mutex;
use rusqlite::{Connection, params, params_from_iter};
use serde::{Deserialize, Serialize};
use tauri::{State, Manager, Emitter, AppHandle};

#[derive(Debug, Serialize, Deserialize)]
pub struct Article {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub content: String,
    pub url: String,
    pub source: String,
    pub category: String,
    pub published_at: String,
    pub fetched_at: String,
    pub heat_score: f64,
    pub is_read: bool,
    pub is_bookmarked: bool,
    pub image_url: String,
}

#[derive(Debug, Serialize)]
pub struct CrawlResult {
    pub inserted: usize,
    pub failed_sources: usize,
}

// Struct for crawled article data (passed between fetch and store)
struct CrawledArticle {
    title: String,
    url: String,
    content: String,
    published_at: String,
    image_url: Option<String>,
}

#[derive(Debug)]
pub struct DbState {
    pub conn: Mutex<Connection>,
}

fn get_db_path() -> Result<String, String> {
    let app_dir = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| "Cannot determine home directory")?;
    let db_dir = format!("{}/.newsagregator", app_dir);

    // Create directory if it doesn't exist
    std::fs::create_dir_all(&db_dir)
        .map_err(|e| format!("Failed to create directory {}: {}", db_dir, e))?;

    Ok(format!("{}/news.db", db_dir))
}

pub fn init_db() -> Result<Connection, rusqlite::Error> {
    let db_path = get_db_path().map_err(|e| rusqlite::Error::ToSqlConversionFailure(e.into()))?;
    let db = Connection::open(&db_path)?;

    // Create articles table if not exists
    db.execute(
        "CREATE TABLE IF NOT EXISTS articles (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            summary TEXT,
            content TEXT,
            url TEXT UNIQUE NOT NULL,
            source TEXT,
            category TEXT,
            published_at TEXT,
            fetched_at TEXT,
            heat_score REAL DEFAULT 0,
            is_read INTEGER DEFAULT 0,
            is_bookmarked INTEGER DEFAULT 0,
            image_url TEXT
        )",
        [],
    )?;

    // Create sources table if not exists
    db.execute(
        "CREATE TABLE IF NOT EXISTS sources (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            url TEXT NOT NULL,
            source_type TEXT NOT NULL,
            is_active INTEGER DEFAULT 1
        )",
        [],
    )?;

    // Create FTS table for full-text search
    db.execute(
        "CREATE VIRTUAL TABLE IF NOT EXISTS articles_fts USING fts5(
            title, summary, content,
            tokenize = 'unicode61'
        )",
        [],
    )?;

    // Seed default sources if table is empty
    let count: i32 = db.query_row("SELECT COUNT(*) FROM sources", [], |row| row.get(0)).unwrap_or(0);
    if count == 0 {
        seed_default_sources(&db)?;
    }

    Ok(db)
}

fn seed_default_sources(conn: &Connection) -> Result<(), rusqlite::Error> {
    let default_sources = vec![
        // International - AI/Tech - Using verified working RSS feeds
        ("Hacker News Frontpage", "https://hnrss.org/frontpage", "RSS", true),
        ("Hacker News AI", "https://hnrss.org/newest?q=AI+OR+machine+learning+OR+GPT+OR+LLM", "RSS", true),

        // GitHub trending pages (using web scraping)
        ("GitHub Trending (all)", "https://github.com/trending", "WEB", true),
        ("GitHub Trending Python", "https://github.com/trending/python", "WEB", true),
        ("GitHub Trending TypeScript", "https://github.com/trending/typescript", "WEB", true),
        ("GitHub Trending Rust", "https://github.com/trending/rust", "WEB", true),

        // Tech news RSS feeds (reliable sources)
        ("Dev.to AI Tag", "https://dev.to/feed/tag/ai", "RSS", true),
        ("Reddit MachineLearning", "https://www.reddit.com/r/MachineLearning/.rss", "RSS", true),

        // Additional reliable AI/Tech RSS feeds
        ("The Verge AI", "https://www.theverge.com/ai-ml/rss", "RSS", true),
        ("Ars Technica AI", "https://arstechnica.com/ai/feed/", "RSS", true),
        ("TechCrunch AI", "https://techcrunch.com/category/artificial-intelligence/feed/", "RSS", true),

        // Chinese tech sites (reliable sources)
        ("OSChina 资讯", "https://www.oschina.net/news/rss", "RSS", true),
        ("V2EX 技术新穗", "https://www.v2ex.com/index.xml", "RSS", true),
        ("InfoQ 中文", "https://www.infoq.cn/feed", "RSS", true),
    ];

    let mut stmt = conn.prepare(
        "INSERT INTO sources (id, name, url, source_type, is_active) VALUES (?, ?, ?, ?, ?)"
    )?;

    for (i, (name, url, source_type, is_active)) in default_sources.iter().enumerate() {
        stmt.execute(params![format!("source_{}", i), name, url, source_type, if *is_active { 1 } else { 0 }])?;
    }

    Ok(())
}

#[tauri::command]
async fn health() -> Result<String, String> {
    Ok("OK".to_string())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListQuery {
    pub page: Option<usize>,
    pub page_size: usize,
    pub category: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ListResponse {
    pub items: Vec<Article>,
    pub total: i64,
    pub page: usize,
    pub page_size: usize,
}

#[tauri::command]
async fn articles_list(
    state: State<'_, DbState>,
    query: ListQuery,
) -> Result<ListResponse, String> {
    let conn = state.conn.lock().map_err(|e| format!("db lock poisoned: {}", e))?;

    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size;
    let offset = (page - 1) * page_size;

    // Build query conditions
    let mut where_clause = String::new();
    let mut params_vec: Vec<String> = Vec::new();

    if let Some(cat) = &query.category {
        if cat != "all" {
            where_clause.push_str(" WHERE category = ?1");
            params_vec.push(cat.clone());
        }
    }

    // Count total
    let count_query = format!("SELECT COUNT(*) FROM articles{}", where_clause);
    let total: i64 = conn.query_row(&count_query, params_from_iter(params_vec.iter()), |row| row.get(0))
        .unwrap_or(0);

    // Get articles
    let list_query = format!(
        "SELECT id, title, summary, content, url, source, category, published_at, fetched_at, heat_score, is_read, is_bookmarked, image_url
         FROM articles{}
         ORDER BY published_at DESC, fetched_at DESC
         LIMIT ?{} OFFSET ?{}",
        where_clause,
        params_vec.len() + 1,
        params_vec.len() + 2
    );

    let page_size_param = page_size as i64;
    let offset_param = offset as i64;
    let mut list_params: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|s| s as &dyn rusqlite::ToSql).collect();
    list_params.push(&page_size_param);
    list_params.push(&offset_param);

    let mut stmt = conn.prepare(&list_query)
        .map_err(|e| format!("prepare failed: {}", e))?;

    let articles: Vec<Article> = stmt.query_map(list_params.as_slice(), |row| {
        let is_read_val: i32 = row.get(10)?;
        let is_bookmarked_val: i32 = row.get(11)?;
        let image_url: Option<String> = row.get(12)?;
        Ok(Article {
            id: row.get(0)?,
            title: row.get(1)?,
            summary: row.get(2)?,
            content: row.get(3)?,
            url: row.get(4)?,
            source: row.get(5)?,
            category: row.get(6)?,
            published_at: row.get(7)?,
            fetched_at: row.get(8)?,
            heat_score: row.get(9)?,
            is_read: is_read_val > 0,
            is_bookmarked: is_bookmarked_val > 0,
            image_url: image_url.unwrap_or_default(),
        })
    }).map_err(|e| format!("query failed: {}", e))?
    .into_iter()
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| format!("collect failed: {}", e))?;

    Ok(ListResponse {
        items: articles,
        total,
        page,
        page_size,
    })
}

#[derive(Debug, Serialize)]
pub struct CleanupResult {
    pub deleted: i32,
}

#[tauri::command]
async fn cleanup_old_articles(state: State<'_, DbState>) -> Result<CleanupResult, String> {
    let conn = state.conn.lock().map_err(|_| "db lock poisoned".to_string())?;
    let max_articles = 300i64;

    let total: i64 = conn.query_row(
        "SELECT COUNT(*) FROM articles",
        [],
        |row| row.get::<_, i64>(0)
    ).map_err(|e| format!("query count failed: {e}"))?;

    if total <= max_articles {
        return Ok(CleanupResult { deleted: 0 });
    }

    let to_delete = total - max_articles;
    let mut stmt = conn.prepare(
        "SELECT rowid FROM articles WHERE is_bookmarked = 0 ORDER BY fetched_at ASC LIMIT ?1"
    ).map_err(|e| format!("prepare cleanup query failed: {e}"))?;

    let mut deleted_count: i32 = 0;
    {
        let mut rows = stmt.query(params![to_delete])
            .map_err(|e| format!("query rows failed: {e}"))?;

        while let Some(row) = rows.next().map_err(|e| format!("next row failed: {e}"))? {
            let rowid: i64 = row.get::<_, i64>(0).map_err(|e| e.to_string())?;
            conn.execute("DELETE FROM articles_fts WHERE rowid = ?1", params![rowid])
                .map_err(|e| format!("delete from fts failed: {e}"))?;
            conn.execute("DELETE FROM articles WHERE rowid = ?1", params![rowid])
                .map_err(|e| format!("delete from articles failed: {e}"))?;
            deleted_count += 1;
        }
    }
    drop(stmt);

    Ok(CleanupResult { deleted: deleted_count })
}

// Search articles
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchQuery {
    pub keyword: String,
}

#[tauri::command]
async fn search_query(state: State<'_, DbState>, query: SearchQuery) -> Result<Vec<Article>, String> {
    let keyword = query.keyword;
    let conn = state.conn.lock().map_err(|e| format!("db lock poisoned: {}", e))?;

    let query = format!(
        "SELECT a.id, a.title, a.summary, a.content, a.url, a.source, a.category, a.published_at, a.fetched_at, a.heat_score, a.is_read, a.is_bookmarked, a.image_url
         FROM articles a
         INNER JOIN articles_fts fts ON a.rowid = fts.rowid
         WHERE articles_fts MATCH ?1
         ORDER BY a.published_at DESC
         LIMIT 100"
    );

    let mut stmt = conn.prepare(&query)
        .map_err(|e| format!("prepare failed: {}", e))?;

    let search_term = format!("{}*", keyword);

    let articles: Vec<Article> = stmt.query_map([search_term], |row| {
        let is_read_val: i32 = row.get(10)?;
        let is_bookmarked_val: i32 = row.get(11)?;
        let image_url: Option<String> = row.get(12)?;
        Ok(Article {
            id: row.get(0)?,
            title: row.get(1)?,
            summary: row.get(2)?,
            content: row.get(3)?,
            url: row.get(4)?,
            source: row.get(5)?,
            category: row.get(6)?,
            published_at: row.get(7)?,
            fetched_at: row.get(8)?,
            heat_score: row.get(9)?,
            is_read: is_read_val > 0,
            is_bookmarked: is_bookmarked_val > 0,
            image_url: image_url.unwrap_or_default(),
        })
    }).map_err(|e| format!("query failed: {}", e))?
    .into_iter()
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| format!("collect failed: {}", e))?;

    Ok(articles)
}

// Toggle bookmark
#[derive(Debug, Serialize, Deserialize)]
pub struct BookmarkPayload {
    pub id: String,
    pub value: bool,
}

#[tauri::command]
async fn article_bookmark(state: State<'_, DbState>, payload: BookmarkPayload) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| format!("db lock poisoned: {}", e))?;
    conn.execute(
        "UPDATE articles SET is_bookmarked = ?1 WHERE id = ?2",
        params![if payload.value { 1 } else { 0 }, payload.id]
    ).map_err(|e| format!("update failed: {}", e))?;
    Ok(())
}

// Mark as read
#[derive(Debug, Serialize, Deserialize)]
pub struct MarkReadPayload {
    pub id: String,
    #[allow(dead_code)]
    pub value: bool,
}

#[tauri::command]
async fn article_mark_read(state: State<'_, DbState>, payload: MarkReadPayload) -> Result<(), String> {
    let conn = state.conn.lock().map_err(|e| format!("db lock poisoned: {}", e))?;
    conn.execute(
        "UPDATE articles SET is_read = 1 WHERE id = ?1",
        params![payload.id]
    ).map_err(|e| format!("update failed: {}", e))?;
    Ok(())
}

// Manual add article
#[derive(Debug, Serialize, Deserialize)]
pub struct ManualAddPayload {
    pub url: String,
}

#[tauri::command]
async fn manual_add(state: State<'_, DbState>, payload: ManualAddPayload) -> Result<Article, String> {
    // Normalize URL
    let normalized_url = normalize_url(&payload.url);

    // Check if article already exists
    {
        let conn = state.conn.lock().map_err(|e| format!("db lock: {}", e))?;
        let exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM articles WHERE url = ?1)",
            params![normalized_url],
            |row| row.get(0)
        ).unwrap_or(false);

        if exists {
            return Err("该链接已存在".to_string());
        }
    }

    // Fetch page content
    let use_proxy = !is_chinese_site(&payload.url);
    let client = create_http_client(use_proxy)?;
    let response = client
        .get(&payload.url)
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| format!("获取页面失败: {}", e))?;

    let html = response.text().await
        .map_err(|e| format!("读取内容失败: {}", e))?;

    // Parse HTML to extract title and content
    let document = scraper::Html::parse_document(&html);

    // Extract title - try <title>, <h1>, og:title
    let title = document
        .select(&scraper::Selector::parse("title").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().trim().to_string())
        .or_else(|| {
            document
                .select(&scraper::Selector::parse("meta[property='og:title']").unwrap())
                .next()
                .and_then(|el| el.value().attr("content"))
                .map(|s| s.to_string())
        })
        .or_else(|| {
            document
                .select(&scraper::Selector::parse("h1").unwrap())
                .next()
                .map(|el| el.text().collect::<String>().trim().to_string())
        })
        .unwrap_or_else(|| "未知标题".to_string());

    // Extract description/content - try meta description, og:description
    let content = document
        .select(&scraper::Selector::parse("meta[name='description']").unwrap())
        .next()
        .and_then(|el| el.value().attr("content"))
        .map(|s| s.to_string())
        .or_else(|| {
            document
                .select(&scraper::Selector::parse("meta[property='og:description']").unwrap())
                .next()
                .and_then(|el| el.value().attr("content"))
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "手动添加的文章".to_string());

    // Generate summary
    let summary = make_zh_brief(&title, &content, "手动添加");

    // Extract image URL
    let image_url = document
        .select(&scraper::Selector::parse("meta[property='og:image']").unwrap())
        .next()
        .and_then(|el| el.value().attr("content"))
        .unwrap_or("")
        .to_string();

    // Insert into database
    let conn = state.conn.lock().map_err(|e| format!("db lock: {}", e))?;

    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO articles (id, title, summary, content, url, source, category, published_at, fetched_at, image_url)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![id, title, summary, content, normalized_url, "手动添加", "Tech", &now, &now, image_url]
    ).map_err(|e| format!("插入失败: {}", e))?;

    // Get the integer rowid for FTS
    let rowid: i64 = conn.last_insert_rowid();

    // Insert into FTS table
    conn.execute(
        "INSERT INTO articles_fts (rowid, title, summary, content) VALUES (?1, ?2, ?3, ?4)",
        params![rowid, title, summary, content]
    ).map_err(|e| format!("FTS 插入失败: {}", e))?;

    Ok(Article {
        id,
        title,
        summary,
        content,
        url: normalized_url,
        source: "手动添加".to_string(),
        category: "Tech".to_string(),
        published_at: now.clone(),
        fetched_at: now,
        heat_score: 0.0,
        is_read: false,
        is_bookmarked: false,
        image_url,
    })
}

// Settings
#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub theme: String,
    pub ai_model: String,
    pub ai_base_url: String,
    pub ai_api_key: String,
    pub ai_summary_enabled: bool,
}

#[tauri::command]
async fn settings_get(state: State<'_, DbState>) -> Result<Settings, String> {
    let conn = state.conn.lock().map_err(|e| format!("db lock poisoned: {}", e))?;

    // Create settings table if not exists
    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT
        )",
        [],
    ).map_err(|e| format!("create table failed: {}", e))?;

    // Get settings from DB or use defaults
    let theme = get_setting(&conn, "theme", "auto")?;
    let ai_model = get_setting(&conn, "ai_model", "")?;
    let ai_base_url = get_setting(&conn, "ai_base_url", "")?;
    let ai_api_key = get_setting(&conn, "ai_api_key", "")?;
    let ai_summary_enabled = get_setting(&conn, "ai_summary_enabled", "true")? == "true";

    // Fallback to environment variables if database is empty
    let ai_model = if ai_model.is_empty() {
        std::env::var("AI_MODEL").unwrap_or_else(|_| "qwen3-max".to_string())
    } else {
        ai_model
    };
    let ai_base_url = if ai_base_url.is_empty() {
        std::env::var("AI_BASE_URL").unwrap_or_default()
    } else {
        ai_base_url
    };
    let ai_api_key = if ai_api_key.is_empty() {
        std::env::var("AI_API_KEY").unwrap_or_default()
    } else {
        ai_api_key
    };

    Ok(Settings {
        theme,
        ai_model,
        ai_base_url,
        ai_api_key,
        ai_summary_enabled,
    })
}

#[tauri::command]
async fn settings_update(state: State<'_, DbState>, payload: Settings) -> Result<Settings, String> {
    let settings = payload;
    let conn = state.conn.lock().map_err(|e| format!("db lock poisoned: {}", e))?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT)",
        [],
    ).map_err(|e| format!("create table failed: {}", e))?;

    set_setting(&conn, "theme", &settings.theme)?;
    set_setting(&conn, "ai_model", &settings.ai_model)?;
    set_setting(&conn, "ai_base_url", &settings.ai_base_url)?;
    set_setting(&conn, "ai_api_key", &settings.ai_api_key)?;
    set_setting(&conn, "ai_summary_enabled", &settings.ai_summary_enabled.to_string())?;

    Ok(settings)
}

fn get_setting(conn: &Connection, key: &str, default: &str) -> Result<String, String> {
    match conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        params![key],
        |row| row.get::<_, String>(0)
    ) {
        Ok(val) => Ok(val),
        Err(_) => Ok(default.to_string()),
    }
}

fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<(), String> {
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
        params![key, value]
    ).map_err(|e| format!("insert failed: {}", e))?;
    Ok(())
}

// AI summarize - calls OpenAI-compatible API
#[tauri::command]
async fn ai_summarize(state: State<'_, DbState>, content: String) -> Result<String, String> {
    // Get settings from database first, then fallback to environment variables
    let (base_url, api_key, model) = {
        let conn = state.conn.lock().map_err(|e| format!("db lock: {}", e))?;
        let db_base_url = get_setting(&conn, "ai_base_url", "").ok().filter(|s| !s.is_empty());
        let db_api_key = get_setting(&conn, "ai_api_key", "").ok().filter(|s| !s.is_empty());
        let db_model = get_setting(&conn, "ai_model", "").ok().filter(|s| !s.is_empty());

        // Try database first, then environment variables
        let base_url = db_base_url.or_else(|| std::env::var("AI_BASE_URL").ok())
            .ok_or_else(|| "请先在设置中配置 AI API Base URL".to_string())?;
        let api_key = db_api_key.or_else(|| std::env::var("AI_API_KEY").ok())
            .ok_or_else(|| "请先在设置中配置 AI API Key".to_string())?;
        let model = db_model.or_else(|| std::env::var("AI_MODEL").ok())
            .unwrap_or_else(|| "qwen3-max".to_string());

        (base_url, api_key, model)
    };

    // Build request - AI APIs usually need proxy for international services
    // But if using Chinese AI services (like DashScope), they work without proxy
    let client = create_http_client(true)?;
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

    let body = serde_json::json!({
        "model": model,
        "messages": [
            {"role": "system", "content": "请用中文总结以下内容，控制在100字以内，突出重点信息。"},
            {"role": "user", "content": content}
        ],
        "max_tokens": 200
    });

    // Send request with timeout
    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| format!("API 请求失败: {}", e))?;

    // Check response status
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        return Err(format!("API 返回错误 ({}): {}", status, error_text));
    }

    // Parse response
    let json: serde_json::Value = response.json().await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    json["choices"][0]["message"]["content"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "API 响应格式错误".to_string())
}

// Progress update structs
#[derive(Debug, Serialize, Clone)]
struct SummaryUpdateStartEvent {
    total: usize,
}

#[derive(Debug, Serialize, Clone)]
struct SummaryUpdateProgressEvent {
    current: usize,
    total: usize,
    title: String,
    updated: usize,
}

#[derive(Debug, Serialize, Clone)]
struct SummaryUpdateCompleteEvent {
    total_updated: usize,
    total_processed: usize,
}

// Batch regenerate summaries
#[tauri::command]
async fn articles_regenerate_summaries(
    state: State<'_, DbState>,
    app: AppHandle,
) -> Result<usize, String> {
    // Check if AI summarization is enabled and configured (from environment variables or database)
    let ai_config = {
        let conn = state.conn.lock().map_err(|_| "db lock poisoned".to_string())?;
        let db_base_url = get_setting(&conn, "ai_base_url", "").ok().filter(|s| !s.is_empty());
        let db_api_key = get_setting(&conn, "ai_api_key", "").ok().filter(|s| !s.is_empty());
        let db_model = get_setting(&conn, "ai_model", "").ok().filter(|s| !s.is_empty());

        let base_url = db_base_url.or_else(|| std::env::var("AI_BASE_URL").ok());
        let api_key = db_api_key.or_else(|| std::env::var("AI_API_KEY").ok());
        let model = db_model.or_else(|| std::env::var("AI_MODEL").ok()).unwrap_or_else(|| "qwen3-max".to_string());

        if let (Some(url), Some(key)) = (base_url, api_key) {
            Some((url, key, model))
        } else {
            None
        }
    };

    if ai_config.is_none() {
        return Err("请先在设置中配置 AI API (Base URL 和 API Key)，或确保 .env 文件中有正确的配置".to_string());
    }

    // Collect all articles with template summaries that need regeneration
    let articles = {
        let conn = state.conn.lock().map_err(|_| "db lock poisoned".to_string())?;
        let mut stmt = conn.prepare(
            "SELECT id, title, content FROM articles WHERE summary LIKE '%这篇英文资讯围绕%' OR summary IS NULL OR summary = ''"
        ).map_err(|e| format!("prepare failed: {e}"))?;

        let result: Vec<(String, String, String)> = stmt.query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
            ))
        }).map_err(|e| format!("query failed: {e}"))?
        .into_iter()
        .filter_map(Result::ok)
        .collect();

        drop(stmt);
        drop(conn);
        result
    };

    let total = articles.len();
    let mut updated = 0;

    // Emit start event
    let start_payload = SummaryUpdateStartEvent { total };
    let _ = app.emit("app://summaries-update:start", start_payload);

    for (index, (id, title, content)) in articles.into_iter().enumerate() {
        let current = index + 1;

        // Emit progress event
        let progress_payload = SummaryUpdateProgressEvent {
            current,
            total,
            title: title.clone(),
            updated,
        };
        let _ = app.emit("app://summaries-update:progress", progress_payload);

        // Generate new summary using AI
        let new_summary = if let Some((ref base_url, ref api_key, ref model)) = ai_config {
            // Create a new HTTP client for each request
            let http_client = create_http_client(true)?;
            match generate_ai_summary(&Some(http_client), base_url, api_key, model, &title, &content).await {
                Ok(ai_summary) => ai_summary,
                Err(e) => {
                    eprintln!("AI summary failed for '{}', using template: {}", title, e);
                    make_zh_brief(&title, &content, "批量更新")
                }
            }
        } else {
            make_zh_brief(&title, &content, "批量更新")
        };

        // Update database - need to acquire lock again
        {
            let conn = state.conn.lock().map_err(|_| "db lock poisoned".to_string())?;
            conn.execute(
                "UPDATE articles SET summary = ?1 WHERE id = ?2",
                params![new_summary, id]
            ).map_err(|e| format!("update failed: {e}"))?;
        } // conn is dropped here

        updated += 1;

        // Emit updated progress
        let progress_payload = SummaryUpdateProgressEvent {
            current,
            total,
            title: title.clone(),
            updated,
        };
        let _ = app.emit("app://summaries-update:progress", progress_payload);

        // Rate limiting between AI calls
        if ai_config.is_some() {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }

    // Emit complete event
    let complete_payload = SummaryUpdateCompleteEvent {
        total_updated: updated,
        total_processed: total,
    };
    let _ = app.emit("app://summaries-update:complete", complete_payload);

    Ok(updated)
}

use reqwest;

// Crawler implementation to fetch from RSS/API sources
#[tauri::command]
async fn crawler_run_once(state: State<'_, DbState>) -> Result<CrawlResult, String> {
    // Get active sources from database
    let sources_data = {
        let conn = state.conn.lock().map_err(|e| format!("db lock poisoned: {}", e))?;

        let mut stmt = conn.prepare(
            "SELECT name, url, source_type FROM sources WHERE is_active = 1 LIMIT 20"
        ).map_err(|e| format!("prepare sources query failed: {}", e))?;

        let sources: Vec<(String, String, String)> = stmt
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                ))
            })
            .map_err(|e| format!("query sources failed: {}", e))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("collect sources failed: {}", e))?;

        sources
    }; // Release the lock before async operations

    // Check if AI summarization is enabled and configured (from environment variables)
    let ai_config = {
        let ai_base_url = std::env::var("AI_BASE_URL").unwrap_or_default();
        let ai_api_key = std::env::var("AI_API_KEY").unwrap_or_default();
        let ai_model = std::env::var("AI_MODEL").unwrap_or_else(|_| "qwen3-max".to_string());

        if !ai_base_url.is_empty() && !ai_api_key.is_empty() {
            Some((ai_base_url, ai_api_key, ai_model))
        } else {
            None
        }
    };

    let mut failed_sources_count = 0;

    // Fetch articles from all sources and generate summaries
    let mut articles_to_insert: Vec<(String, CrawledArticle, String)> = Vec::new();

    for (source_name, source_url, source_type) in sources_data {
        let result = fetch_articles_from_source(&source_name, &source_url, &source_type).await;

        match result {
            Ok(articles) => {
                for article in articles {
                    // Generate summary using AI if configured, otherwise use template
                    let summary = if let Some((ref base_url, ref api_key, ref model)) = ai_config {
                        let http_client = create_http_client(true)?;
                        match generate_ai_summary(&Some(http_client), base_url, api_key, model, &article.title, &article.content).await {
                            Ok(ai_summary) => ai_summary,
                            Err(e) => {
                                eprintln!("AI summary failed for '{}', using template: {}", article.title, e);
                                make_zh_brief(&article.title, &article.content, &source_name)
                            }
                        }
                    } else {
                        make_zh_brief(&article.title, &article.content, &source_name)
                    };

                    articles_to_insert.push((source_name.clone(), article, summary));

                    // Rate limiting between AI calls
                    if ai_config.is_some() {
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    }
                }
            },
            Err(e) => {
                eprintln!("Failed to fetch from source '{}': {}", source_name, e);
                failed_sources_count += 1;
            }
        }
    }

    // Now store all articles using the shared connection
    let mut inserted_total = 0;
    {
        let conn = state.conn.lock().map_err(|e| format!("db lock poisoned: {}", e))?;

        for (source_name, article, summary) in articles_to_insert {
            // Check if article already exists
            let exists: bool = conn.query_row(
                "SELECT EXISTS(SELECT 1 FROM articles WHERE url = ?1)",
                params![&article.url],
                |row| row.get(0)
            ).unwrap_or(false);

            if !exists {
                let id = uuid::Uuid::new_v4().to_string();
                let category = categorize_source(&source_name);

                // Insert into articles table
                conn.execute(
                    "INSERT INTO articles (id, title, summary, content, url, source, category, published_at, fetched_at, image_url)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                    params![
                        &id,
                        &article.title,
                        &summary,
                        &article.content,
                        &article.url,
                        &source_name,
                        &category,
                        &article.published_at,
                        &chrono::Utc::now().to_rfc3339(),
                        &article.image_url.unwrap_or_default()
                    ]
                ).map_err(|e| format!("Insert article failed: {}", e))?;

                // Get the integer rowid for FTS
                let rowid: i64 = conn.last_insert_rowid();

                // Insert into FTS table using integer rowid
                conn.execute(
                    "INSERT INTO articles_fts (rowid, title, summary, content) VALUES (?1, ?2, ?3, ?4)",
                    params![rowid, &article.title, &summary, &article.content]
                ).map_err(|e| format!("Insert into FTS failed: {}", e))?;

                inserted_total += 1;
            }
        }
    }

    // Clean up old articles after crawling
    let _cleanup_result = cleanup_old_articles(state).await?;

    Ok(CrawlResult {
        inserted: inserted_total,
        failed_sources: failed_sources_count
    })
}

// Fetch articles from a source, returning data without database operations
async fn fetch_articles_from_source(source_name: &str, url: &str, source_type: &str) -> Result<Vec<CrawledArticle>, String> {
    match source_type {
        "RSS" => fetch_rss_feed(source_name, url).await,
        "WEB" => {
            // Check if this is a GitHub trending URL
            if url.contains("github.com/trending") {
                fetch_github_trending(source_name, url).await
            } else {
                fetch_web_page(source_name, url).await
            }
        },
        _ => Ok(Vec::new())
    }
}

// Create HTTP client with optional proxy for international sites
fn create_http_client(use_proxy: bool) -> Result<reqwest::Client, String> {
    let mut builder = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .connect_timeout(std::time::Duration::from_secs(10))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36");

    if use_proxy {
        // Check for proxy in environment variables or use default
        if let Ok(proxy_url) = std::env::var("HTTP_PROXY")
            .or_else(|_| std::env::var("http_proxy"))
            .or_else(|_| std::env::var("HTTPS_PROXY"))
            .or_else(|_| std::env::var("https_proxy"))
        {
            match reqwest::Proxy::all(&proxy_url) {
                Ok(proxy) => {
                    builder = builder.proxy(proxy);
                    println!("Using proxy: {}", proxy_url);
                }
                Err(e) => eprintln!("Failed to configure proxy '{}': {}", proxy_url, e),
            }
        } else {
            // Try default proxy at 127.0.0.1:7897 (common Clash proxy)
            let default_proxy = "http://127.0.0.1:7897";
            match reqwest::Proxy::all(default_proxy) {
                Ok(proxy) => {
                    builder = builder.proxy(proxy);
                    println!("Using default proxy: {}", default_proxy);
                }
                Err(_) => {
                    println!("No proxy configured (default proxy not available)");
                }
            }
        }
    }

    builder.build().map_err(|e| format!("Failed to create HTTP client: {}", e))
}

// Check if URL or source name indicates a Chinese domestic site (no proxy needed)
fn is_chinese_site(url: &str) -> bool {
    let chinese_domains = [
        ".cn",               // .cn domains
        "oschina.net",       // OSChina
        "v2ex.com",          // V2EX
        "leiphone.com",      // 雷锋网
        "tmtpost.com",       // 钛媒体
        "36kr.com",          // 36氪
        "jiqizhixin.com",    // 机器之心
        "qbitai.com",        // 量子位
        "zhidx.com",         // 智东西
        "infoq.cn",          // InfoQ中文
        "hellogithub.com",   // HelloGitHub
        "csdn.net",          // CSDN
        "juejin.cn",         // 掘金
        "segmentfault.com",  // SegmentFault
    ];

    let url_lower = url.to_lowercase();
    chinese_domains.iter().any(|domain| url_lower.contains(domain))
}

// Fetch RSS feed and return articles (no database operations)
async fn fetch_rss_feed(source_name: &str, url: &str) -> Result<Vec<CrawledArticle>, String> {
    let use_proxy = !is_chinese_site(url);
    let client = create_http_client(use_proxy)?;

    // Add headers to mimic a real browser request - let reqwest handle compression automatically
    let response = client
        .get(url)
        .header("Accept", "application/rss+xml, application/xml, text/xml;q=0.9, */*;q=0.8")
        .header("Accept-Language", "en-US,en;q=0.9")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .header("Referer", "https://www.google.com/")
        .header("sec-ch-ua", "\"Not_A Brand\";v=\"8\", \"Chromium\";v=\"120\"")
        .header("sec-ch-ua-mobile", "?0")
        .header("sec-ch-ua-platform", "\"Windows\"")
        .send().await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let content = response.text().await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    // Check if response is HTML instead of XML/RSS (common anti-bot response)
    let content_lower = content.to_lowercase();
    if content_lower.contains("<!doctype html")
        || content_lower.contains("just a moment")
        || content_lower.contains("checking your browser")
        || content_lower.contains("access denied")
        || content_lower.contains("<title>404")
        || content_lower.contains("page not found")
        || content_lower.contains("<html") {
        eprintln!("RSS feed {} returned HTML instead of RSS/XML (possible anti-bot protection), skipping: {}", source_name, url);
        return Ok(Vec::new());
    }

    // Attempt to parse as RSS
    let channel = match rss::Channel::read_from(content.as_bytes()) {
        Ok(channel) => channel,
        Err(e) => {
            eprintln!("Could not parse RSS for source: {} - Error: {:?}. Content preview: {:.100}", source_name, e, content);
            return Ok(Vec::new());
        }
    };

    let mut articles = Vec::new();

    // Limit to 12 items per source
    for item in channel.items().iter().take(12) {
        if let Some(title) = item.title() {
            if let Some(link) = item.link() {
                let description = item.description().unwrap_or("No description available").to_string();
                let content = description.clone();
                let pub_date = item.pub_date().unwrap_or("");
                let normalized_date = normalize_datetime(pub_date);
                let image_url = item.enclosure().map(|e| e.url.to_string());

                articles.push(CrawledArticle {
                    title: title.to_string(),
                    url: normalize_url(link),
                    content,
                    published_at: normalized_date,
                    image_url,
                });
            }
        }
    }

    Ok(articles)
}

// Fetch web page and return articles (no database operations)
async fn fetch_web_page(_source_name: &str, url: &str) -> Result<Vec<CrawledArticle>, String> {
    let use_proxy = !is_chinese_site(url);
    let client = create_http_client(use_proxy)?;

    let response = client
        .get(url)
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
        .send().await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let content = response.text().await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    let document = scraper::Html::parse_document(&content);
    let selector = scraper::Selector::parse("a").map_err(|e| format!("Invalid selector: {}", e))?;

    let mut articles = Vec::new();
    let now = chrono::Utc::now().to_rfc3339();

    for element in document.select(&selector).take(12) {
        if let Some(href) = element.value().attr("href") {
            if href.starts_with("http") {
                let abs_url = href.to_string();
                let title = element.text().collect::<Vec<_>>().join(" ").trim().to_string();

                if !title.is_empty() {
                    let content = "Web-scraped content".to_string();

                    articles.push(CrawledArticle {
                        title: title.clone(),
                        url: normalize_url(&abs_url),
                        content,
                        published_at: now.clone(),
                        image_url: None,
                    });
                }
            }
        }
    }

    Ok(articles)
}

// Fetch GitHub trending projects with quality filtering
async fn fetch_github_trending(source_name: &str, url: &str) -> Result<Vec<CrawledArticle>, String> {
    let use_proxy = true; // GitHub needs proxy for international access
    let client = create_http_client(use_proxy)?;

    let response = client
        .get(url)
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .send().await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    let content = response.text().await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    // First pass: extract all project data from trending page
    let mut projects_data: Vec<(String, String, String, String, u32)> = Vec::new();

    {
        let document = scraper::Html::parse_document(&content);

        // GitHub trending article selector
        let article_selector = scraper::Selector::parse("article.Box-row").map_err(|e| format!("Invalid selector: {}", e))?;

        for row in document.select(&article_selector) {
            if let Some(name_element) = row.select(&scraper::Selector::parse("h2 a").unwrap()).next() {
                let project_url = name_element.value().attr("href").unwrap_or("").to_string();
                let project_name = name_element.text().collect::<String>().trim().to_string();

                let description = row
                    .select(&scraper::Selector::parse("p").unwrap())
                    .next()
                    .map(|el| el.text().collect::<String>().trim().to_string())
                    .unwrap_or_default();

                let language = row
                    .select(&scraper::Selector::parse("span[itemprop='programmingLanguage']").unwrap())
                    .next()
                    .map(|el| el.text().collect::<String>().trim().to_string())
                    .unwrap_or_default();

                let stars_text = row
                    .select(&scraper::Selector::parse("a[href$='/stargazers']").unwrap())
                    .next()
                    .map(|el| el.text().collect::<String>().trim().to_string())
                    .unwrap_or_default();
                let stars = parse_number(&stars_text);

                projects_data.push((project_url, project_name, description, language, stars));
            }
        }
        drop(document); // Explicitly drop document before await
    }

    let mut articles = Vec::new();
    let now = chrono::Utc::now();

    // Second pass: fetch project pages and apply quality filter
    for (project_url, project_name, description, language, stars) in projects_data {
        if project_url.is_empty() {
            continue;
        }

        // Get project created time by fetching project page
        let full_url = format!("https://github.com{}", project_url);
        let created_at = fetch_github_project_created(&client, &full_url).await;

        // Quality filter based on project age
        // - New projects (< 2 weeks): stars > 20k
        // - Recent projects (< 2 months): stars > 30k
        // - Old projects (>= 2 months): stars > 10k
        let is_quality = if let Some(created_time) = created_at {
            let age_days = (now - created_time).num_days();
            if age_days < 14 {
                stars > 20000
            } else if age_days < 60 {
                stars > 30000
            } else {
                stars > 10000
            }
        } else {
            // Cannot determine age, use default threshold
            stars > 10000
        };

        if is_quality {
            let language_info = if !language.is_empty() { format!(" [{}]", language) } else { String::new() };
            let title = format!("{}{}", project_name, language_info);
            let content = if !description.is_empty() { description.clone() } else { "GitHub trending project".to_string() };

            articles.push(CrawledArticle {
                title,
                url: normalize_url(&full_url),
                content,
                published_at: now.to_rfc3339(),
                image_url: None,
            });
        }
    }

    println!("GitHub Trending [{}]: found {} quality projects (filtered)", source_name, articles.len());
    Ok(articles)
}

// Fetch GitHub project page to get created time
async fn fetch_github_project_created(client: &reqwest::Client, url: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    let response = client
        .get(url)
        .header("Accept", "text/html")
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .ok()?;

    let content = response.text().await.ok()?;
    let document = scraper::Html::parse_document(&content);

    // Look for relative time element with created date
    // GitHub uses <relative-time> elements for timestamps
    for time_elem in document.select(&scraper::Selector::parse("relative-time").unwrap()) {
        if let Some(datetime) = time_elem.value().attr("datetime") {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(datetime) {
                return Some(dt.with_timezone(&chrono::Utc));
            }
        }
    }

    // Alternative: look for time element with specific class
    for time_elem in document.select(&scraper::Selector::parse("time").unwrap()) {
        if let Some(datetime) = time_elem.value().attr("datetime") {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(datetime) {
                return Some(dt.with_timezone(&chrono::Utc));
            }
        }
    }

    None
}

// Parse number from GitHub's format (e.g., "1.2k" -> 1200, "15.5k" -> 15500)
fn parse_number(text: &str) -> u32 {
    let text = text.replace(',', "").replace(' ', "");
    if text.to_lowercase().ends_with('k') {
        let num: f64 = text[..text.len()-1].parse().unwrap_or(0.0);
        (num * 1000.0) as u32
    } else {
        text.parse().unwrap_or(0)
    }
}

// Helper function to normalize URLs (as mentioned in the documentation)
fn normalize_url(url: &str) -> String {
    let mut url_clean = url.trim().to_lowercase();
    if url_clean.ends_with('/') {
        url_clean.pop();
    }
    url_clean
}

// Helper function to categorize source
fn categorize_source(source_name: &str) -> String {
    if source_name.contains("GitHub") {
        "GitHub".to_string()
    } else if source_name.contains("AI") || source_name.contains("人工") || source_name.contains("智能") {
        "AI".to_string()
    } else {
        "Tech".to_string()
    }
}

// Helper function to make Chinese brief summary (template as fallback)
fn make_zh_brief(title: &str, content: &str, _source: &str) -> String {
    let safe_content = if content.chars().count() > 20 {
        content.chars().take(20).collect::<String>()
    } else {
        content.to_string()
    };
    format!("这篇英文资讯围绕「{}」展开，介绍了{}等关键内容。建议点击标题查看原文。", title, safe_content)
}

// Generate AI summary with exponential backoff retry
async fn generate_ai_summary(
    client: &Option<reqwest::Client>,
    base_url: &str,
    api_key: &str,
    model: &str,
    title: &str,
    content: &str,
) -> Result<String, String> {
    let client = client.as_ref().ok_or_else(|| "HTTP client not initialized".to_string())?;
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

    // Truncate content to avoid token limits (use chars to avoid UTF-8 boundary issues)
    let truncated_content = if content.chars().count() > 3000 {
        content.chars().take(3000).collect::<String>()
    } else {
        content.to_string()
    };

    let body = serde_json::json!({
        "model": model,
        "messages": [
            {"role": "system", "content": "请用中文总结以下内容，控制在 100 字以内，突出重点信息。"},
            {"role": "user", "content": format!("标题：{}\n\n内容：{}", title, truncated_content)}
        ],
        "max_tokens": 200
    });

    // Exponential backoff retry (3 attempts: 2s, 4s, 8s delays)
    let mut attempts = 0;
    let delays = [2, 4, 8];

    loop {
        attempts += 1;

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await;

        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    let json: serde_json::Value = resp.json().await
                        .map_err(|e| format!("解析响应失败：{}", e))?;

                    if let Some(summary) = json["choices"][0]["message"]["content"].as_str() {
                        return Ok(summary.to_string());
                    } else {
                        return Err("API 响应格式错误".to_string());
                    }
                } else {
                    let status = resp.status();
                    let error_text = resp.text().await.unwrap_or_default();
                    eprintln!("AI API error ({}): {}", status, error_text);

                    if attempts >= 3 {
                        return Err(format!("API 返回错误 ({}): {}", status, error_text));
                    }
                }
            }
            Err(e) => {
                eprintln!("AI request attempt {} failed: {}", attempts, e);

                if attempts >= 3 {
                    return Err(format!("API 请求失败：{}", e));
                }
            }
        }

        // Wait before retry
        if attempts < 3 {
            tokio::time::sleep(tokio::time::Duration::from_secs(delays[attempts - 1])).await;
        }
    }
}

// Helper function to normalize date/time formats to ISO 8601
fn normalize_datetime(date_str: &str) -> String {
    if date_str.is_empty() {
        return chrono::Utc::now().to_rfc3339();
    }

    // Try parsing various formats and convert to ISO 8601
    if let Ok(dt) = chrono::DateTime::parse_from_rfc2822(date_str) {
        return dt.with_timezone(&chrono::Utc).to_rfc3339();
    }
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(date_str) {
        return dt.with_timezone(&chrono::Utc).to_rfc3339();
    }

    // If parsing fails, return current time
    chrono::Utc::now().to_rfc3339()
}

// Open URL in system browser
#[tauri::command]
async fn open_external(url: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &url])
            .spawn()
            .map_err(|e| format!("failed to open url: {}", e))?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("failed to open url: {}", e))?;
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("failed to open url: {}", e))?;
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // Initialize database
            let db = init_db().map_err(|e| format!("Failed to initialize database: {}", e))?;
            app.manage(DbState {
                conn: Mutex::new(db),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            health,
            articles_list,
            cleanup_old_articles,
            search_query,
            article_bookmark,
            article_mark_read,
            manual_add,
            settings_get,
            settings_update,
            ai_summarize,
            articles_regenerate_summaries,
            crawler_run_once,
            open_external,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
