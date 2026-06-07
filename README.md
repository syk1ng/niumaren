# 🐂 牛马人 · 周末值班通知系统

Windows 桌面应用，自动化团队周末值班排班与邮件通知。系统托盘常驻后台，定时检查并提前 2 天发送提醒邮件，自动跳过中国法定节假日。

## 功能

| 功能 | 说明 |
|------|------|
| 👥 人员管理 | 值班人员增删改、排序、启用/禁用 |
| 📧 邮件通知 | SMTP 发送，支持自定义模板变量 |
| 🔄 顺序轮值 | 按人员列表顺序轮流安排周末值班 |
| ⏰ 提前通知 | 提前 2 天自动发送（周六值班→周四通知，周日值班→周五通知） |
| 🎌 节假日跳过 | 自动识别中国法定节假日（timor.tech API + 本地缓存） |
| 🖥️ 系统托盘 | 关闭窗口隐藏到托盘，右键菜单操作 |
| 📊 排班日志 | 排班记录 + 邮件发送日志查看 |

## 界面

5 个标签页：**人员管理** → **邮件配置** → **邮件模板** → **排班表** → **发送日志**

## 下载

[📥 最新版本 v0.1.0](https://github.com/syk1ng/niumaren/releases/latest)

免安装，下载 `niumaren.exe` 直接运行即可。需要 Windows 10+ 和 WebView2 运行时（系统通常已自带）。

## 模板变量

邮件标题和正文支持以下变量：

| 变量 | 说明 |
|------|------|
| `{姓名}` | 值班人员姓名 |
| `{邮箱}` | 值班人员邮箱 |
| `{日期}` | 值班日期（YYYY-MM-DD） |
| `{星期}` | 星期几 |
| `{下一位姓名}` | 下一个值班人姓名 |
| `{下一位日期}` | 下一个值班日期 |

## 技术栈

| 层 | 技术 |
|----|------|
| 桌面框架 | Tauri 2.x |
| 前端 | React 19 + TypeScript + Tailwind CSS |
| 后端 | Rust |
| 数据库 | SQLite (rusqlite) |
| 邮件 | lettre (SMTP) |
| 调度 | tokio |
| 节假日 | timor.tech API + reqwest |
| 日期 | chrono |

## 开发

```bash
# 安装依赖
npm install

# 开发模式（热更新）
npm run tauri dev

# 打包
npm run tauri build
```

## 项目结构

```
niumaren/
├── src/                    # React 前端
│   ├── App.tsx             # 主布局 + 标签页
│   ├── components/         # 5 个标签页组件
│   ├── hooks/              # IPC 调用 Hook
│   └── types/              # TypeScript 类型定义
├── src-tauri/              # Rust 后端
│   └── src/
│       ├── main.rs         # 入口、托盘、窗口事件
│       ├── db.rs           # SQLite 数据库
│       ├── commands.rs     # IPC 命令处理
│       ├── email.rs        # SMTP 邮件 + 模板替换
│       ├── holiday.rs      # 节假日 API + 缓存
│       ├── scheduler.rs    # 定时调度 + 轮值逻辑
│       └── models.rs       # 数据模型
└── index.html
```

## License

MIT
