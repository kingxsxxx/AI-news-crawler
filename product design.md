# AI资讯聚合器 - 产品设计文档

## 1. 产品概述

### 1.1 产品定位
一款面向AI从业者、研究者和爱好者的智能资讯聚合桌面应用，自动追踪AI领域最新动态，提供个性化的内容筛选和深度阅读体验。

### 1.2 核心价值
- **自动化**: 每日自动抓取全网AI相关资讯，无需手动浏览多个网站
- **智能化**: 基于热度算法和AI分类，让用户看到最有价值的内容
- **个性化**: 支持分类筛选、搜索和收藏，打造专属信息流
- **时效性**: 自动清理过时内容，保持信息新鲜度
- **BYOK模式**: 用户自带API Key，自主选择AI模型（GPT-4/Claude等）

---

## 2. 功能模块

### 2.1 内容抓取系统

#### 数据源规划
| 类型 | 数据源 | 抓取频率 | 内容特点 | 抓取方式 |
|------|--------|----------|----------|----------|
| **技术发布** | GitHub Trending, Papers with Code | 每6小时 | 开源项目、模型发布 | API优先 |
| **技术博客** | Google AI Blog, OpenAI Blog, Anthropic Blog | 每12小时 | 官方技术解读 | RSS优先 |
| **学术论文** | ArXiv (cs.AI, cs.CL, cs.CV), HuggingFace Papers | 每日 | 最新研究成果 | API |
| **热门产品** | Product Hunt, BetaList, AI Tools Directory | 每12小时 | AI新产品发布 | Headless渲染 |
| **行业资讯** | TechCrunch AI, VentureBeat, The Verge | 每6小时 | 大厂动态、融资新闻 | RSS/API |
| **创业公司** | Crunchbase, 36kr, 机器之心 | 每日 | 初创公司动态 | RSS/API |
| **社区讨论** | Hacker News, Reddit r/MachineLearning | 每4小时 | 热点讨论 | API |

#### 内容抓取流程
```
定时触发 → 多源并行抓取(带并发控制) → 内容清洗 → 去重检测 → AI分类 → 热度计算 → 入库
```

#### 热度算法（含冷启动优化）
```
热度分 = (浏览数 × 1) + (点赞数 × 3) + (评论数 × 5) + (分享数 × 4) + (来源权重 × 15) - (时间衰减)

时间衰减 = 当前时间 - 发布时间(小时) × 0.5

来源权重说明:
- 官方源(OpenAI/Anthropic/Google): 权重 1.0 (确保权威内容优先展示)
- 顶级学术(ArXiv/顶会): 权重 0.9
- 知名社区(HN/Reddit): 权重 0.8
- 行业媒体: 权重 0.6
- 其他: 权重 0.4
```

### 2.2 内容管理系统

#### 数据模型
```
Article (文章)
├── id: UUID
├── title: String (标题)
├── summary: String (AI生成摘要)
├── content: Text (正文或富文本)
├── readable_content: Text (readability提取的纯净内容)
├── url: String (原文链接)
├── source: String (来源网站)
├── source_weight: Float (来源权重)
├── category: Enum (分类: Tech/Research/Product/Industry/Fun)
├── tags: Array<String> (标签)
├── image_url: String (封面图)
├── author: String (作者)
├── published_at: DateTime (发布时间)
├── fetched_at: DateTime (抓取时间)
├── heat_score: Float (热度分 0-100)
├── view_count: Int (浏览数)
├── click_count: Int (点击数)
├── is_read: Boolean (是否已读)
├── is_bookmarked: Boolean (是否收藏)
├── is_archived: Boolean (是否归档)
└── is_manual: Boolean (是否手动录入)

Source (数据源)
├── id: UUID
├── name: String (名称)
├── url: String (URL)
├── type: Enum (类型: RSS/API/Web/Headless)
├── category: Enum (内容分类)
├── fetch_interval: Int (抓取间隔分钟)
├── last_fetch_at: DateTime
├── is_active: Boolean
├── priority: Int (优先级)
└── weight: Float (来源权重)

Settings (用户设置)
├── id: UUID
├── theme: Enum (主题: light/dark/auto)
├── auto_cleanup_days: Int (自动清理天数, 默认30)
├── notify_new_content: Boolean (新内容通知)
├── preferred_categories: Array<String> (偏好分类)
├── language: String (界面语言)
├── ai_api_key: String (用户自有AI API Key)
├── ai_model: String (首选AI模型)
└── ai_summary_enabled: Boolean (是否启用AI摘要)
```

#### 自动清理策略
- **过期内容**: 超过30天的文章自动归档
- **低热度内容**: 热度<5且超过7天的内容自动清理
- **手动清理**: 用户可一键清理已读内容

#### 手动录入功能
用户可主动粘贴URL，系统自动:
1. 解析URL元数据（标题、摘要、封面图）
2. 提取正文内容（readability算法）
3. AI生成分类和标签
4. 存入个人收藏库

### 2.3 前端展示系统

#### 页面结构
```
App
├── Sidebar (左侧导航)
│   ├── Logo
│   ├── Navigation (Feed/Explore/Saved/History)
│   ├── Categories (分类快捷入口)
│   └── Settings
├── Main Content (主内容区)
│   ├── Header (搜索栏 + 筛选器 + 主题切换)
│   ├── Content Grid (卡片网格布局)
│   └── Article Modal (文章详情弹窗)
└── Status Bar (底部状态栏)
    ├── 最后更新时间
    ├── 今日新文章数
    └── 抓取状态指示器
```

#### 核心页面

**1. 信息流页 (Feed)**
- 瀑布流/网格布局展示文章卡片
- 支持分类筛选和排序
- 无限滚动加载
- 实时热度排行榜

**2. 搜索发现页 (Explore)**
- 类似Bilibili的搜索界面
- 热门搜索推荐
- 高级筛选(时间范围、分类、热度)
- 搜索结果分类展示

**3. 收藏夹 (Saved)**
- 用户收藏的文章
- 手动录入入口
- 支持标签管理
- 导出功能

**4. 历史记录 (History)**
- 阅读历史
- 浏览统计

**5. 设置页 (Settings)**
- 主题设置
- AI配置（BYOK模式）
- 数据源管理
- 通知设置
- 清理策略

### 2.4 搜索功能

#### 搜索能力
- **全文搜索**: 标题、摘要、正文、标签（支持中文分词）
- **智能建议**: 输入时实时显示搜索建议
- **历史记录**: 保存搜索历史
- **筛选器**:
  - 时间范围: 今天/本周/本月/全部
  - 分类: Tech/Research/Product/Industry/Fun
  - 热度: 热门/最新/最多浏览
  - 来源: 多选特定网站

#### 搜索算法
- SQLite FTS (Full-Text Search) + jieba-rs中文分词
- 支持布尔搜索 (AND/OR/NOT)
- 相关性排序 (标题匹配 > 摘要匹配 > 正文匹配)

---

## 3. UI/UX 设计规范

### 3.1 设计理念
- **现代极简**: 留白充足，信息密度适中
- **视觉层次**: 清晰的字体大小和颜色层级
- **流畅动效**: 微交互提升体验
- **深色友好**: 支持深浅主题

### 3.2 色彩系统

```css
/* 主色调 */
--primary: #3B82F6;        /* 蓝色 - 品牌色 */
--primary-hover: #2563EB;

/* 分类色 */
--tech: #8B5CF6;           /* 紫色 - 技术发布 */
--research: #10B981;       /* 绿色 - 学术论文 */
--product: #F59E0B;        /* 橙色 - 产品发布 */
--industry: #EF4444;       /* 红色 - 行业资讯 */
--fun: #EC4899;            /* 粉色 - 趣闻 */

/* 中性色 */
--bg-primary: #FFFFFF;     /* 背景 */
--bg-secondary: #F3F4F6;   /* 次要背景 */
--text-primary: #111827;   /* 主要文字 */
--text-secondary: #6B7280; /* 次要文字 */
--border: #E5E7EB;         /* 边框 */
```

### 3.3 字体规范
- **标题**: Inter / Noto Sans SC, Bold, 20-24px
- **正文**: Inter / Noto Sans SC, Regular, 14-16px
- **辅助**: Inter / Noto Sans SC, Regular, 12-13px

### 3.4 组件规范

**文章卡片**
```
┌──────────────────────────────────────────┐
│  [封面图 16:9]                            │
├──────────────────────────────────────────┤
│  [分类标签] [热度值]              [收藏★]  │
│  文章标题 (2行截断)                         │
│  摘要 (2-3行, 灰色)                        │
│                                          │
│  来源 · 3小时前                    → 阅读  │
└──────────────────────────────────────────┘
尺寸: 宽度自适应, 高度根据内容
圆角: 12px
阴影: 0 1px 3px rgba(0,0,0,0.1)
悬停: 上浮 + 阴影加深
```

**文章详情页**
```
┌──────────────────────────────────────────┐
│  ← 返回                    [收藏] [分享]  │
├──────────────────────────────────────────┤
│  分类标签  热度值                          │
│  文章标题                                  │
│  作者 · 来源 · 发布时间                     │
├──────────────────────────────────────────┤
│  [封面大图]                                │
│                                          │
│  正文内容 (支持Markdown渲染)               │
│                                          │
│  [阅读原文] [相关推荐]                      │
└──────────────────────────────────────────┘
```

**搜索界面 (类似Bilibili)**
```
┌──────────────────────────────────────────┐
│  [🔍 搜索AI论文、开源项目...      ] [搜索] │
├──────────────────────────────────────────┤
│  热门搜索                                  │
│  [#ChatGPT] [#GPT-4] [#Stable Diffusion] │
│                                          │
│  历史记录                                  │
│  [AI Agent] [Transformer]               │
└──────────────────────────────────────────┘
```

**手动录入入口**
```
┌──────────────────────────────────────────┐
│  + 粘贴文章链接                            │
│  支持任意网页，自动提取正文和生成摘要         │
└──────────────────────────────────────────┘
```

---

## 4. 开发路线图

### Phase 1: MVP (2-3周)
- [ ] 项目初始化 (Tauri + React + Tailwind)
- [ ] 数据库设计实现
- [ ] Trait-based爬虫架构
- [ ] 基础爬虫 (3-5个核心数据源，RSS/API优先)
- [ ] 文章列表和详情页
- [ ] 基础搜索功能（含中文分词）
- [ ] 深色/浅色主题

### Phase 2: 功能完善 (1-2周)
- [ ] 扩展数据源 (10+来源)
- [ ] Headless Chrome支持SPA网站
- [ ] 手动录入功能
- [ ] 高级搜索和筛选
- [ ] 收藏和历史功能
- [ ] 热度排行榜
- [ ] 自动清理机制
- [ ] BYOK AI配置
- [ ] 数据导出功能

### Phase 3: 体验优化 (1周)
- [ ] 流畅动效和过渡
- [ ] readability正文提取
- [ ] 离线阅读支持
- [ ] 性能优化 (虚拟滚动)
- [ ] 新内容通知
- [ ] 快捷键支持

### Phase 4: 扩展 (可选)
- [ ] AI智能摘要
- [ ] 个性化推荐
- [ ] 多设备同步
- [ ] 分享功能

---

## 5. 性能指标

| 指标 | 目标值 |
|------|--------|
| 安装包大小 | < 15MB |
| 启动时间 | < 2秒 |
| 首屏加载 | < 1秒 |
| 搜索响应 | < 200ms |
| 同时显示文章 | 1000+ (虚拟滚动) |
| 数据库存储 | 支持10万+文章 |
| 抓取并发 | 最大3个并发（Semaphore控制） |

---

## 6. 风险与应对

| 风险 | 应对策略 |
|------|----------|
| 数据源反爬 | 尊重robots.txt, 合理频率, User-Agent模拟, RSS/API优先 |
| 网站改版导致解析失败 | Trait-based插件化架构，单数据源故障不影响整体 |
| 内容版权问题 | 只存储摘要和链接, 引导到原文阅读 |
| 抓取失败 | 日志记录, 失败重试, 用户可手动刷新 |
| 内容质量 | 来源白名单, 关键词过滤, 用户反馈 |
| SPA网站抓取 | Headless Chrome作为兜底方案 |

---

## 7. 附录

### 7.1 推荐数据源API
- GitHub Trending API: https://github.com/trending (网页抓取)
- ArXiv API: http://export.arxiv.org/api/query
- Hacker News API: https://github.com/HackerNews/API
- Product Hunt API: https://api.producthunt.com/v2/docs

### 7.2 参考设计
- Arc Browser: 侧边栏 + 主内容区布局
- Notion: 简洁的文档卡片设计
- Bilibili: 搜索和推荐界面
- Feedly: RSS阅读器信息流

---

**文档版本**: v2.0
**更新日期**: 2024
**作者**: AI Assistant
