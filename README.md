# AI News Aggregator

一个 AI 资讯聚合桌面应用，使用 Tauri v2 构建。

![Version](https://img.shields.io/badge/version-0.1.0-blue.svg)
![Tauri](https://img.shields.io/badge/Tauri-v2-24C8DB)
![React](https://img.shields.io/badge/React-18.2-61DAFB)
![Rust](https://img.shields.io/badge/Rust-2021-orange)

## 功能特性

- **多源聚合**：自动从多个 RSS  feed、网页和 API 获取 AI 相关资讯
- **本地存储**：使用 SQLite 存储文章，支持全文检索（FTS5）
- **AI 摘要**：支持调用 AI API 生成中文摘要（可选）
- **书签管理**：支持文章收藏和阅读标记
- **跨平台**：支持 Windows、macOS、Linux

## 支持的资讯源

### 国际 AI/科技
- OpenAI Blog
- Google AI Blog
- DeepMind Blog
- Anthropic News
- MIT Tech Review AI
- VentureBeat AI
- Hacker News

### 中文 AI/科技
- 雷锋网 AI
- 钛媒体
- 36 氪
- 机器之心
- 量子位
- 智东西
- InfoQ 中文

### GitHub Trending
- GitHub Trending（全部）
- GitHub Trending AI
- GitHub Trending Python/TypeScript/Rust
- HelloGitHub 月刊

## 环境要求

在开始之前，请确保安装以下依赖：

- [Node.js](https://nodejs.org/) >= 18.x
- [Rust](https://www.rust-lang.org/tools/install) >= 1.70
- npm（随 Node.js 一起安装）

### Windows 额外要求

安装 [Microsoft Visual Studio C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)

## 安装步骤

### 1. 克隆项目

```bash
git clone <your-repo-url>
cd Newsagregator
```

### 2. 安装前端依赖

```bash
npm install
```

### 3. 配置环境变量（可选）

如果需要使用 AI 摘要功能，请复制环境变量配置文件并填写真实的 API 密钥：

```bash
cp .env.example .env
```

编辑 `.env` 文件：

```env
# AI 服务配置（用于生成中文摘要）
AI_BASE_URL=https://dashscope.aliyuncs.com/compatible-mode/v1
AI_MODEL=qwen3-max
AI_API_KEY=your_real_api_key_here
```

> **说明**：
> - 默认使用阿里云 DashScope（通义千问）API
> - 如不配置 AI 密钥，应用将使用模板生成摘要（例如："这篇英文资讯围绕..."）
> - 也可以使用其他 OpenAI 兼容格式的 API 服务

### 4. 运行开发环境

```bash
# 启动完整 Tauri 应用（Rust 后端 + React 前端）
npm run tauri:dev
```

首次运行时会自动创建 SQLite 数据库（存储在系统应用数据目录下的 `news.db`），并种子化默认的资讯源。

### 5. 构建生产版本

```bash
npm run tauri:build
```

构建产物将输出到 `src-tauri/target/release/bundle/` 目录。

## 可用命令

| 命令 | 说明 |
|------|------|
| `npm install` | 安装前端依赖 |
| `npm run dev` | 仅运行 Vite 前端开发服务器 |
| `npm run build` | TypeScript 检查 + Vite 构建（输出到 dist/） |
| `npm run tauri:dev` | 运行完整 Tauri 应用（开发模式） |
| `npm run tauri:build` | 构建生产版本 |
| `cargo check` | 验证 Rust 代码编译（在 src-tauri 目录下） |

## 项目结构

```
Newsagregator/
├── src/                    # React 前端
│   ├── components/         # UI 组件
│   ├── lib/
│   │   └── api.ts         # Tauri 命令调用封装
│   ├── types/             # TypeScript 类型定义
│   ├── App.tsx            # 主应用组件
│   └── main.tsx           # 入口文件
├── src-tauri/
│   ├── src/
│   │   ├── main.rs        # Rust 入口
│   │   └── lib.rs         # Tauri 命令 + 数据库 + 爬虫
│   ├── Cargo.toml         # Rust 依赖配置
│   └── tauri.conf.json    # Tauri 配置
├── .env.example           # 环境变量示例
├── package.json           # Node.js 依赖配置
└── README.md              # 项目文档
```

## 数据库结构

应用使用 SQLite 存储数据，主要表结构：

- **articles** - 文章表（标题、摘要、内容、链接、来源、分类、时间等）
- **articles_fts** - 全文检索虚拟表（FTS5）
- **settings** - 用户设置（主题、AI 配置等）
- **sources** - 资讯源配置

## API 命令

后端提供的 Tauri 命令（通过 `invoke()` 调用）：

| 命令 | 说明 |
|------|------|
| `health` | 健康检查 |
| `articles_list` | 获取文章列表（支持分页和分类过滤） |
| `article_get` | 根据 ID 获取单篇文章 |
| `article_bookmark` | 切换书签状态 |
| `article_mark_read` | 标记已读 |
| `search_query` | 全文搜索（bm25 排序） |
| `manual_add` | 手动添加文章（通过 URL） |
| `crawler_run_once` | 运行一次爬虫 |
| `ai_summarize` | 生成 AI 摘要 |
| `settings_get/update` | 获取/更新设置 |

## 常见问题

### Q: 爬虫运行缓慢或超时？
A: 爬虫默认每次运行最多处理 20 个资讯源。网络状况可能影响速度。AI 摘要和 OG 图片获取已禁用以避免超时。

### Q: AI 摘要不工作？
A: 检查 `.env` 文件中的 `AI_BASE_URL`、`AI_MODEL` 和 `AI_API_KEY` 是否配置正确。如未配置，将使用模板摘要作为降级方案。

### Q: 数据库文件在哪里？
A: 存储在系统应用数据目录下：
- Windows: `%APPDATA%\ai-news-aggregator\news.db`
- macOS: `~/Library/Application Support/ai-news-aggregator/news.db`
- Linux: `~/.config/ai-news-aggregator/news.db`

## License

MIT
