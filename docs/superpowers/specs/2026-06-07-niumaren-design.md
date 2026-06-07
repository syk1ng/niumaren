# 牛马人（NiuMaRen）— 周末值班通知系统 设计文档

> 日期: 2026-06-07 | 状态: Draft

## 1. 概述

### 1.1 项目背景

开发一个 Windows 桌面应用「牛马人」，用于自动化周末值班通知。解决团队周末值班排班、提醒的痛点，避免人工排班遗漏。

### 1.2 核心功能

- **人员管理**：自定义值班人员姓名和邮箱，支持增删改和排序
- **顺序轮值**：按人员列表顺序轮流安排周末值班（一人一天）
- **提前通知**：提前 2 天通过自定义邮箱自动发送通知邮件（周六值班→周四通知，周日值班→周五通知）
- **邮件模板**：支持变量替换模板 `{姓名}` `{日期}` `{星期}` `{下一位姓名}` `{下一位日期}`
- **节假日跳过**：自动识别中国法定节假日，跳过节假日所在的周末天数

### 1.3 技术选型

| 模块 | 技术 | 说明 |
|------|------|------|
| 桌面框架 | Tauri 2.x | Rust 后端 + 系统 WebView，体积小(<5MB)，系统托盘原生支持 |
| 前端 | React + TypeScript + Tailwind CSS | 现代化 UI，类型安全 |
| 数据库 | SQLite (tauri-plugin-sql) | 本地单文件存储，支持结构化查询 |
| 邮件发送 | lettre (Rust crate) | 异步 SMTP，支持 TLS/SSL |
| 节假日 | timor.tech API + 本地缓存 | 免费、稳定、中国法定节假日 |
| 邮件模板 | 变量替换 | `{姓名}` `{日期}` `{星期}` 等占位符 |
| 运行模式 | 系统托盘 + 开机自启 | 最小化到托盘后台运行，定时检查并发送 |

---

## 2. 系统架构

```
┌─────────────────────────────────────────┐
│              前端 (React + TS)           │
│  人员管理 │ 邮件配置 │ 模板编辑          │
│  排班表   │ 发送日志                     │
└──────────────┬──────────────────────────┘
               │ IPC invoke
┌──────────────▼──────────────────────────┐
│         Tauri Rust Backend               │
│                                           │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ │
│  │ 命令处理  │ │ 定时调度  │ │ 邮件发送  │ │
│  │ (CRUD)   │ │ (tokio)  │ │ (lettre)  │ │
│  └──────────┘ └──────────┘ └──────────┘ │
│  ┌──────────┐ ┌──────────────────────┐   │
│  │ 节假日    │ │ 系统托盘              │   │
│  │ (API+缓存)│ │ (tray-icon)          │   │
│  └──────────┘ └──────────────────────┘   │
└──────────────┬──────────────────────────┘
               │
     ┌─────────┼─────────┐
     ▼         ▼         ▼
  ┌──────┐ ┌──────┐ ┌──────┐
  │SQLite│ │ SMTP │ │节假日│
  │      │ │Server│ │ API  │
  └──────┘ └──────┘ └──────┘
```

---

## 3. 数据模型

### 3.1 数据库表结构

**personnel（人员表）**

| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER PK | 自增主键 |
| name | TEXT NOT NULL | 姓名 |
| email | TEXT NOT NULL | 邮箱地址 |
| sort_order | INTEGER | 排序序号（决定轮值顺序） |
| active | INTEGER DEFAULT 1 | 是否启用（0=禁用，1=启用） |
| created_at | TEXT | 创建时间 |

**schedule（排班表）**

| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER PK | 自增主键 |
| person_id | INTEGER FK | 值班人员 ID |
| duty_date | TEXT NOT NULL | 值班日期（YYYY-MM-DD） |
| is_holiday | INTEGER DEFAULT 0 | 是否被标记为节假日跳过 |
| notified | INTEGER DEFAULT 0 | 是否已发送通知（0=否，1=是） |
| notified_at | TEXT | 通知发送时间 |
| created_at | TEXT | 记录创建时间 |

**email_log（邮件日志表）**

| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER PK | 自增主键 |
| schedule_id | INTEGER FK | 关联排班记录 |
| recipient | TEXT | 收件人邮箱 |
| subject | TEXT | 邮件标题（变量替换后） |
| status | TEXT | 发送状态：success / failed |
| error_msg | TEXT | 失败原因 |
| sent_at | TEXT | 发送时间 |

**settings（设置表）**

| 字段 | 类型 | 说明 |
|------|------|------|
| key | TEXT PK | 设置键 |
| value | TEXT | 设置值 |

### 3.2 设置项

| Key | 说明 | 示例值 |
|-----|------|--------|
| smtp_host | SMTP 服务器 | smtp.qq.com |
| smtp_port | SMTP 端口 | 465 |
| smtp_username | 发件邮箱 | zhangsan@qq.com |
| smtp_password | SMTP 授权码（加密存储） | *** |
| smtp_use_tls | 使用 TLS | true |
| email_subject_template | 邮件标题模板 | 【值班通知】{日期} {星期} |
| email_body_template | 邮件正文模板 | Hi {姓名}：本周末... |
| sender_name | 发件人名称 | 值班系统 |
| auto_start | 开机自启 | true |
| last_person_index | 上次轮值到第几人 | 2 |
| holiday_cache | 节假日缓存 JSON | {...} |
| holiday_cache_year | 缓存年份 | 2026 |

### 3.3 邮件模板变量

| 变量 | 说明 | 示例 |
|------|------|------|
| `{姓名}` | 值班人员姓名 | 张三 |
| `{邮箱}` | 值班人员邮箱 | zhangsan@qq.com |
| `{日期}` | 值班日期 | 2026-06-14 |
| `{星期}` | 星期几 | 星期日 |
| `{下一位姓名}` | 下一个值班人 | 李四 |
| `{下一位日期}` | 下一个值班日期 | 2026-06-20 |

---

## 4. 核心业务流程

### 4.1 排班轮值算法

- 每天安排 1 人，周六和周日各安排不同的人
- 轮值索引全局递增：周六用 N，周日用 N+1，下周六用 N+2...
- 只有成功发送通知后索引才 +1（未发送不消耗轮值名额）
- 禁用的（active=0）人员自动跳过

```
function getNextDutyPerson(lastIndex, personnelList):
    activeList = personnelList.filter(p => p.active)
    nextIndex = (lastIndex + 1) % len(activeList)
    return activeList[nextIndex]
```

### 4.2 通知触发逻辑

```
定时任务（每小时执行一次）:
  今天 = today()
  周六 = getNextSaturday(today)
  周日 = getNextSunday(today)

  // 检查周六通知（周四触发）
  if today == 周六 - 2天:
    if not isHoliday(周六):
      person = getNextDutyPerson()
      sendEmail(person, 周六)
      recordSchedule(person, 周六)

  // 检查周日通知（周五触发）
  if today == 周日 - 2天:
    if not isHoliday(周日):
      person = getNextDutyPerson()
      sendEmail(person, 周日)
      recordSchedule(person, 周日)
```

### 4.3 节假日判断

```
function isHoliday(date):
    // 1. 先查本地缓存
    if cached and cacheYear == date.year:
        return date in holidays

    // 2. 调用 API 更新缓存
    holidays = fetchHolidays(date.year)  // timor.tech API
    updateCache(holidays, date.year)
    return date in holidays
```

### 4.4 邮件发送

```
function sendEmail(person, date):
    template = loadTemplate()
    subject = replaceVars(template.subject, person, date)
    body = replaceVars(template.body, person, date)

    result = smtp.send(
        from: settings.smtp_username,
        to: person.email,
        subject: subject,
        body: body
    )

    logEmail(schedule_id, result)
```

---

## 5. 界面设计

### 5.1 主窗口（Tab 切换）

```
┌─────────────────────────────────────────┐
│ 🐂 牛马人 · 值班助手          ─ □ ✕   │
├─────────────────────────────────────────┤
│ [人员管理] [邮件配置] [邮件模板]       │
│ [排班表] [发送日志]                     │
├─────────────────────────────────────────┤
│                                         │
│         （Tab 内容区域）                 │
│                                         │
├─────────────────────────────────────────┤
│ 状态栏：下次发送：2026-06-11 · 运行中   │
└─────────────────────────────────────────┘
```

### 5.2 各 Tab 内容

**人员管理**：
- 列表展示（序号、姓名、邮箱、启用状态）
- 增/删/改按钮
- 拖拽或上下箭头调整顺序
- 当前轮值位置标记

**邮件配置**：
- SMTP 服务器、端口、用户名（邮箱）、授权码（密码框）
- TLS/SSL 开关
- 发件人名称
- 「测试发送」按钮（发送到指定邮箱验证配置）

**邮件模板**：
- 邮件标题输入框（带变量提示）
- 邮件正文文本框（多行，带变量提示）
- 变量列表参考卡片
- 「预览」按钮（变量替换后效果预览）

**排班表**：
- 日历/列表视图切换
- 显示历史排班和未来排班
- 节假日标记（灰色/删除线）
- 手动「跳过」/「调整」按钮

**发送日志**：
- 列表：日期、收件人、标题、状态、时间
- 失败项标红，支持「重新发送」
- 筛选：全部/成功/失败

### 5.3 系统托盘

```
托盘图标：🐂（或自定义 ico）
右键菜单：
  ├── 打开主面板
  ├── 查看排班表
  ├── 手动发送通知
  ├── ──────────
  └── 退出
```

---

## 6. Tauri 项目结构

```
niumaren/
├── src-tauri/                  # Rust 后端
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── icons/                  # 应用图标
│   └── src/
│       ├── main.rs             # 入口：托盘、窗口、插件注册
│       ├── commands.rs         # IPC 命令（CRUD 操作）
│       ├── scheduler.rs        # 定时调度（tokio::time）
│       ├── email.rs            # 邮件发送（lettre）
│       ├── holiday.rs          # 节假日 API + 缓存
│       ├── db.rs               # SQLite 初始化和操作
│       └── models.rs           # 数据结构定义
├── src/                        # React 前端
│   ├── App.tsx                 # 主布局（Tab 切换 + 状态栏）
│   ├── main.tsx                # React 入口
│   ├── components/
│   │   ├── PersonnelTab.tsx    # 人员管理
│   │   ├── EmailConfigTab.tsx  # 邮件配置
│   │   ├── TemplateTab.tsx     # 邮件模板
│   │   ├── ScheduleTab.tsx     # 排班表
│   │   └── LogTab.tsx          # 发送日志
│   ├── hooks/
│   │   └── useTauriInvoke.ts   # IPC 调用封装
│   └── types/
│       └── index.ts            # TypeScript 类型定义
├── package.json
├── tsconfig.json
├── vite.config.ts
├── tailwind.config.js
└── index.html
```

---

## 7. 关键依赖

### Rust (Cargo.toml)

| Crate | 版本 | 用途 |
|-------|------|------|
| tauri | 2.x | 桌面框架 |
| tauri-plugin-sql | 2.x | SQLite 支持 |
| tauri-plugin-shell | 2.x | 开机自启（shell） |
| tauri-plugin-notification | 2.x | 系统通知 |
| lettre | 0.11 | SMTP 邮件 |
| tokio | 1.x | 异步运行时 + 定时器 |
| serde / serde_json | 1.x | 序列化 |
| rusqlite | 0.31 | SQLite 驱动 |
| reqwest | 0.12 | HTTP 请求（节假日 API） |
| chrono | 0.4 | 日期处理 |

### 前端 (package.json)

| Package | 用途 |
|---------|------|
| react / react-dom | UI 框架 |
| typescript | 类型检查 |
| tailwindcss | 样式 |
| vite | 构建工具 |
| @tauri-apps/api | Tauri 前端 API |
| @tauri-apps/plugin-sql | 前端 SQL 调用（可选） |
| lucide-react | 图标库 |

---

## 8. 错误处理

### 8.1 邮件发送失败

- 重试策略：失败后间隔 30 分钟重试，最多 3 次
- 日志记录：记录详细错误信息到 email_log
- 用户提示：托盘弹出系统通知提醒用户检查配置

### 8.2 节假日 API 不可用

- 降级策略：使用本地缓存数据
- 缓存过期处理：标记为"待更新"，使用上一年的日期作为参考
- 用户提示：界面显示"节假日数据可能不准确"

### 8.3 错过通知窗口

- 如果应用在周四/周五未运行（错过了发通知时间），下次启动时检测未通知的排班记录
- 对于已过期的未通知排班（日期已过），标记为"已错过"，不补发
- 对于仍在未来 2 天内的排班，立即补发通知

### 8.4 应用异常

- Rust panic 由 Tauri 框架捕获
- 前端错误边界组件防止白屏
- 托盘模式崩溃时自动重启

---

## 9. 安全考虑

- SMTP 授权码：使用系统凭据管理器存储或加密后存入 SQLite
- 邮件模板：防止用户输入恶意 HTML（纯文本发送，不做 HTML 渲染）
- 前端输入校验：邮箱格式、必填字段验证

---

## 10. 待定项

- 节假日 API 备选方案（如 timor.tech 不稳定，可切换到天行数据 API）
- 打包和自动更新方案（Tauri updater plugin）
- 是否需要支持多团队/多排班组
