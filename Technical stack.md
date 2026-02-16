# AI资讯聚合器 - 技术架构文档

## 1. 技术选型

### 1.1 核心栈

| 层级 | 技术 | 说明 |
|------|------|------|
| **桌面框架** | Tauri v2 | Rust + Webview, 轻量高性能 |
| **前端框架** | React 18 + TypeScript | 组件化开发，类型安全 |
| **UI组件库** | Shadcn/ui + TailwindCSS | 精美组件，灵活定制 |
| **状态管理** | Zustand | 轻量级状态管理 |
| **后端语言** | Rust | 高性能爬虫和数据处理 |
| **数据库** | SQLite + rusqlite | 轻量本地存储 |

### 1.2 爬虫与解析

| 组件 | 技术 | 说明 |
|------|------|------|
| **HTTP客户端** | reqwest | Rust异步HTTP |
| **HTML解析** | scraper | Rust网页解析 |
| **RSS解析** | rss | RSS feed解析 |
| **动态渲染** | headless_chrome | 处理SPA网站（如Product Hunt） |
| **正文提取** | readability | 提取纯净正文内容 |

### 1.3 搜索与AI

| 组件 | 技术 | 说明 |
|------|------|------|
| **全文搜索** | SQLite FTS5 | 全文检索 |
| **中文分词** | jieba-rs | 中文搜索支持 |
| **定时任务** | tokio-cron-scheduler | 异步定时任务 |
| **并发控制** | tokio::sync::Semaphore | 限制抓取并发数 |

---

## 2. 架构设计

### 2.1 整体架构图

```
┌─────────────────────────────────────────────────────────────┐
│                        前端层 (React)                        │
├─────────────┬─────────────┬─────────────┬───────────────────┤
│   Feed页    │  Search页   │  Saved页    │    Settings页     │
├─────────────┴─────────────┴─────────────┴───────────────────┤
│                      Tauri Bridge                            │
├─────────────────────────────────────────────────────────────┤
│                        后端层 (Rust)                         │
├─────────────┬─────────────┬─────────────┬───────────────────┤
│  API Routes │   Crawler   │  Database   │    Scheduler      │
│   (Axum)    │ (插件化架构) │  (SQLite)   │   (定时任务)      │
└─────────────┴─────────────┴─────────────┴───────────────────┘
                              │
                    ┌─────────┴─────────┐
                    ▼                   ▼
              ┌──────────┐       ┌──────────┐
              │  数据源1  │       │  数据源N  │
              │ GitHub等 │       │ ArXiv等  │
              └──────────┘       └──────────┘
```

### 2.2 爬虫插件化架构（Trait-based）

```rust
// 核心 Trait 定义
pub trait SourceCrawler: Send + Sync {
    fn name(&self) -> &str;
    fn source_type(&self) -> SourceType;
    fn fetch_interval(&self) -> Duration;

    async fn fetch(&self) -> Result<Vec<RawContent>, Error>;
    async fn parse(&self, content: RawContent) -> Result<ParsedArticle, Error>;
}

// 数据源类型枚举
pub enum SourceType {
    RSS,        // RSS Feed
    API,        // JSON API
    Web,        // 静态网页抓取
    Headless,   // 动态渲染（SPA）
}

// 爬虫管理器
pub struct CrawlerManager {
    crawlers: Vec<Box<dyn SourceCrawler>>,
    semaphore: Arc<Semaphore>,  // 并发控制
}
```

### 2.3 数据源实现示例

```
src/crawler/sources/
├── mod.rs              # 模块导出
├── github.rs           # GitHub Trending (Web/API)
├── arxiv.rs            # ArXiv论文 (API)
├── hackernews.rs       # Hacker News (API)
├── reddit.rs           # Reddit (API)
├── producthunt.rs      # Product Hunt (Headless)
├── rss_generic.rs      # 通用RSS源
└── ...
```

---

## 3. 数据流

### 3.1 抓取流程

```
Scheduler触发
    │
    ▼
┌─────────────────────────────────────────────────────────┐
│                    CrawlerManager                        │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │  Source A   │  │  Source B   │  │  Source C   │     │
│  │   (RSS)     │  │   (API)     │  │ (Headless)  │     │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘     │
│         │                │                │            │
│         └────────────────┼────────────────┘            │
│                          ▼                             │
│              Semaphore(最大3并发)                       │
└──────────────────────────┬─────────────────────────────┘
                           │
           ┌───────────────┼───────────────┐
           ▼               ▼               ▼
      ┌─────────┐    ┌─────────┐    ┌─────────┐
      │ Parser  │    │ Parser  │    │ Parser  │
      │ (RSS)   │    │ (JSON)  │    │ (HTML)  │
      └────┬────┘    └────┬────┘    └────┬────┘
           │               │               │
           └───────────────┼───────────────┘
                           ▼
              ┌─────────────────────┐
              │   Deduplicator      │
              │  (URL/标题相似度)    │
              └──────────┬──────────┘
                         ▼
              ┌─────────────────────┐
              │   AI Classifier     │
              │  (分类/标签/摘要)    │
              └──────────┬──────────┘
                         ▼
              ┌─────────────────────┐
              │   Heat Calculator   │
              │   (热度分计算)       │
              └──────────┬──────────┘
                         ▼
                    ┌──────────┐
                    │ Database │
                    │ (SQLite) │
                    └──────────┘
```

### 3.2 展示流程

```
User Action
    │
    ▼
React Component
    │
    ▼
Zustand Store
    │
    ▼
Tauri API (invoke)
    │
    ▼
Rust Command Handler
    │
    ▼
Database Query (rusqlite)
    │
    ▼
JSON Response
    │
    ▼
React State Update → UI Re-render
```

### 3.3 搜索流程

```
Search Input
    │
    ▼
Debounce (300ms)
    │
    ▼
Jieba-rs 分词
    │
    ▼
SQLite FTS5 Query
    │
    ▼
Rank by Relevance
    │
    ▼
Filter (category/time/source)
    │
    ▼
Display Results
```

---

## 4. 项目结构

```
ai-news-aggregator/
├── src-tauri/                      # Rust后端
│   ├── src/
│   │   ├── main.rs                 # 应用入口
│   │   ├── lib.rs                  # 模块导出
│   │   ├── commands/               # Tauri命令
│   │   │   ├── article.rs          # 文章相关API
│   │   │   ├── search.rs           # 搜索API
│   │   │   ├── source.rs           # 数据源API
│   │   │   └── settings.rs         # 设置API
│   │   ├── crawler/                # 爬虫模块
│   │   │   ├── mod.rs              # 爬虫管理器
│   │   │   ├── fetcher.rs          # HTTP请求封装
│   │   │   ├── parser.rs           # 内容解析
│   │   │   ├── scheduler.rs        # 定时任务
│   │   │   └── sources/            # 各数据源实现
│   │   │       ├── mod.rs
│   │   │       ├── github.rs
│   │   │       ├── arxiv.rs
│   │   │       ├── hackernews.rs
│   │   │       ├── reddit.rs
│   │   │       ├── producthunt.rs
│   │   │       └── rss_generic.rs
│   │   ├── db/                     # 数据库模块
│   │   │   ├── mod.rs
│   │   │   ├── models.rs           # 数据模型
│   │   │   ├── repository.rs       # 数据操作
│   │   │   └── migrations/         # 数据库迁移
│   │   ├── search/                 # 搜索模块
│   │   │   ├── mod.rs
│   │   │   ├── tokenizer.rs        # jieba分词
│   │   │   └── fts.rs              # FTS查询
│   │   └── utils/                  # 工具函数
│   │       ├── heat_calculator.rs  # 热度计算
│   │       ├── classifier.rs       # AI分类
│   │       ├── readability.rs      # 正文提取
│   │       └── cleanup.rs          # 清理任务
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── src/                            # React前端
│   ├── main.tsx                    # React入口
│   ├── App.tsx                     # 根组件
│   ├── components/                 # 通用组件
│   │   ├── ui/                     # 基础UI组件
│   │   ├── layout/                 # 布局组件
│   │   │   ├── Sidebar.tsx
│   │   │   ├── Header.tsx
│   │   │   └── MainLayout.tsx
│   │   ├── article/                # 文章组件
│   │   │   ├── ArticleCard.tsx
│   │   │   ├── ArticleGrid.tsx
│   │   │   └── ArticleDetail.tsx
│   │   ├── search/                 # 搜索组件
│   │   │   ├── SearchBar.tsx
│   │   │   ├── SearchFilters.tsx
│   │   │   └── SearchSuggestions.tsx
│   │   └── common/                 # 通用组件
│   │       ├── CategoryBadge.tsx
│   │       ├── HeatIndicator.tsx
│   │       ├── EmptyState.tsx
│   │       └── ManualEntry.tsx     # 手动录入组件
│   ├── pages/                      # 页面组件
│   │   ├── FeedPage.tsx
│   │   ├── SearchPage.tsx
│   │   ├── SavedPage.tsx
│   │   ├── HistoryPage.tsx
│   │   └── SettingsPage.tsx
│   ├── hooks/                      # 自定义Hooks
│   │   ├── useArticles.ts
│   │   ├── useSearch.ts
│   │   └── useSettings.ts
│   ├── store/                      # Zustand状态管理
│   │   ├── articleStore.ts
│   │   ├── searchStore.ts
│   │   └── settingsStore.ts
│   ├── types/                      # TypeScript类型
│   │   └── index.ts
│   ├── lib/                        # 工具函数
│   │   ├── utils.ts
│   │   └── api.ts                  # Tauri API封装
│   └── styles/                     # 样式文件
│       ├── globals.css
│       └── tailwind.config.js
│
├── public/                         # 静态资源
│   └── images/
├── package.json
├── tsconfig.json
└── README.md
```

---

## 5. 关键实现细节

### 5.1 并发控制

```rust
use tokio::sync::Semaphore;
use std::sync::Arc;

pub struct CrawlerManager {
    semaphore: Arc<Semaphore>,
}

impl CrawlerManager {
    pub fn new() -> Self {
        // 最多3个并发，保护源站负载
        Self {
            semaphore: Arc::new(Semaphore::new(3)),
        }
    }

    pub async fn fetch_all(&self) {
        let futures = self.crawlers.iter().map(|crawler| {
            let permit = self.semaphore.clone().acquire_owned().await.unwrap();
            async move {
                let _permit = permit;
                crawler.fetch().await
            }
        });

        futures::future::join_all(futures).await;
    }
}
```

### 5.2 RSS/API优先策略

```rust
impl SourceCrawler for ArxivCrawler {
    fn source_type(&self) -> SourceType {
        SourceType::API  // 优先使用API
    }

    async fn fetch(&self) -> Result<Vec<RawContent>, Error> {
        // 使用ArXiv官方API
        let url = "http://export.arxiv.org/api/query";
        let response = reqwest::get(url).await?;
        // ...
    }
}

impl SourceCrawler for ProductHuntCrawler {
    fn source_type(&self) -> SourceType {
        SourceType::Headless  // SPA网站需要Headless
    }

    async fn fetch(&self) -> Result<Vec<RawContent>, Error> {
        // 使用headless_chrome渲染
        let browser = headless_chrome::Browser::default()?;
        // ...
    }
}
```

### 5.3 中文搜索实现

```rust
use jieba_rs::Jieba;

pub struct ChineseTokenizer {
    jieba: Jieba,
}

impl ChineseTokenizer {
    pub fn tokenize(&self, text: &str) -> Vec<String> {
        self.jieba.cut(text, true)
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }

    // 入库前预分词
    pub fn prepare_for_fts(&self, text: &str) -> String {
        let tokens = self.tokenize(text);
        tokens.join(" ")  // 空格分隔供FTS使用
    }
}
```

### 5.4 正文提取（Readability）

```rust
pub fn extract_readable_content(html: &str) -> Result<String, Error> {
    let document = readability::extractor::extract(
        &mut html.as_bytes(),
        &url::Url::parse("http://example.com")?
    )?;

    Ok(document.text)
}
```

---

## 6. 依赖配置

### 6.1 Cargo.toml (Rust)

```toml
[dependencies]
# 核心
tauri = { version = "2.0", features = [] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }

# 数据库
rusqlite = { version = "0.30", features = ["bundled", "chrono"] }

# HTTP
reqwest = { version = "0.11", features = ["json"] }

# HTML解析
scraper = "0.18"

# RSS
rss = "2.0"

# 动态渲染（可选，按需启用）
headless_chrome = { version = "1.0", optional = true }

# 中文分词
jieba-rs = "0.6"

# 正文提取
readability = "0.3"

# 定时任务
tokio-cron-scheduler = "0.10"

# 工具
chrono = "0.4"
anyhow = "1.0"
thiserror = "1.0"
```

### 6.2 package.json (Node)

```json
{
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "@tauri-apps/api": "^2.0.0",
    "zustand": "^4.4.0",
    "tailwindcss": "^3.3.0",
    "@radix-ui/react-*": "latest",
    "class-variance-authority": "latest",
    "clsx": "latest",
    "tailwind-merge": "latest"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2.0.0",
    "@types/react": "^18.2.0",
    "@types/react-dom": "^18.2.0",
    "typescript": "^5.0.0",
    "vite": "^4.4.0"
  }
}
```

---

## 7. 性能优化策略

| 优化点 | 策略 |
|--------|------|
| 安装包大小 | Tauri原生体积优势，剔除无用依赖 |
| 启动速度 | 延迟加载非核心模块，数据库连接池 |
| 搜索性能 | FTS5索引，jieba预分词，结果缓存 |
| 列表渲染 | 虚拟滚动(virtuoso)，分页加载 |
| 抓取性能 | 并发控制(Semaphore)，增量更新 |
| 内存占用 | 流式处理大页面，定期清理缓存 |

---

**文档版本**: v2.0
**更新日期**: 2024
**作者**: AI Assistant
