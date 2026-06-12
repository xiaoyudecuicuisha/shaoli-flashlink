# 绍理闪连 V3.0

校园网络自动登录与学生效率工具集。基于 Tauri 2 + Vue 3 + Rust 构建，体积小、启动快、资源占用低。

## 功能一览

### 校园网络

| 功能 | 说明 |
|------|------|
| **一键登录** | 支持移动/联通/电信三大运营商，自动保存账号密码（AES-256-GCM 加密），支持开机自启+断网自动重连 |
| **网络诊断** | 连通性检测、公网 IP 查询、延迟测试、下载测速、DNS 诊断、VPN/代理检测、Hosts 修复 |
| **校园服务** | 格式转换（Word/WPS）、校园服务入口、校园站点导航 |

### 效率工具

| 功能 | 说明 |
|------|------|
| **白板** | 集成 Drawnix 绘图引擎，支持多白板管理、自由画布、思维导图、流程图 |
| **系统清理** | 规则驱动的文件扫描、NTFS MFT 大文件快速查找、软件卸载引擎、空文件夹清理 |
| **网课助手** | 超星平台窗口管理、多格式题库导入（TXT/JSON/XLSX/DOCX）、模糊匹配自动答题 |

### 实验室

| 功能 | 说明 |
|------|------|
| **QQ 空间** | QR 扫码登录，获取历史动态（说说+评论），导出 Excel/HTML/图片 |
| **桌面宠物** | GIF 宠物、自定义上传、心情系统、右键菜单、跨模块联动（网络/网课/QQ空间/转换状态感知） |

## 技术栈

| 层 | 技术 |
|------|------|
| 框架 | Tauri 2 (Rust 后端 + WebView 前端) |
| 前端 | Vue 3 + Vite + Pinia + Vue Router |
| 样式 | 自研设计系统（CSS Variables + Double-Bezel 架构）|
| 后端 | Rust (tokio async runtime) |
| 构建 | GitHub Actions CI/CD + NSIS 安装包 |

## 系统要求

- Windows 10 / 11 (x64)
- 无需额外运行时依赖

## 安装

从 [Releases](https://github.com/xiaoyudecuicuisha/shaoli-flashlink/releases) 下载最新版 `.exe` 安装包，运行安装即可。

应用内置自动更新，启动后在「更多」页面点击「检查更新」可获取新版本。

## 开发

### 环境准备

- [Node.js](https://nodejs.org/) 20+
- [pnpm](https://pnpm.io/) 10+
- [Rust](https://www.rust-lang.org/tools/install) (stable)

### 启动开发服务器

```bash
# 终端 1：启动前端开发服务器
cd ui && pnpm dev

# 终端 2：启动 Tauri 应用
npx @tauri-apps/cli dev
```

### 构建发布版

```bash
# 自动编译前端 + Rust，生成 NSIS 安装包
npx @tauri-apps/cli build
```

## 项目结构

```
src-tauri/src/
├── main.rs              入口点（CLI/GUI 双模式）
├── lib.rs               Tauri 应用构建、命令注册、托盘
├── commands.rs          Tauri 命令路由层（90+ 命令）
├── srun/                深澜校园网认证协议
├── network/             网络诊断模块
├── system/              配置、自启、Hosts、提权
├── qzone/               QQ 空间获取与导出
├── whiteboard/          白板集成
├── cleaner/             系统清理引擎
├── course/              网课助手
├── pet/                 桌面宠物
└── utils/               错误处理

ui/src/
├── views/               页面组件（10 个视图）
├── components/          设计系统组件
├── types/               TypeScript 类型定义
└── styles/              CSS 设计系统
```

## 数据存储

| 路径 | 内容 |
|------|------|
| `{exe目录}/config.json` | 应用配置（账号、主题、自启等） |
| `~/.shaoli/logs/` | 运行日志（7 天自动清理） |
| `~/.shaoli/crash/` | 崩溃报告 |
| `~/.shaoli/pets/` | 桌面宠物配置 |
| `~/.shaoli/school.json` | 学校认证配置 |

## 许可证

本项目仅供学习交流使用。
