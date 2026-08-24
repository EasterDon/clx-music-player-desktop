# 一梦音乐播放器 (Clx Music Player)

> 仓库目录名：`clx-music-player-desktop`（双壳桌面客户端，非仅 Tauri）

一个桌面端音乐播放器，除了常规听歌，顺便帮你弹。

---

## 项目结构

一个项目两个壳，共用同一套后端接口：

|  | 技术栈 | 说明 |
|--|--------|------|
| **app-tauri** | Tauri v2 + Vue 3 + TypeScript | 常规桌面应用 |
| **app-gpui** | Rust + [GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui) + [Kira](https://github.com/tesselode/kira) | 纯 Rust 原生实现，无 WebView |

## 功能

- 在线播放 MP3（HTTP Range 分片边下边播）、歌词滚动高亮、歌单按名称筛选
- 演奏模式：选一首歌按 F1 开始"弹奏"（模拟键盘输入），F2 停止，可后台输入
- 宽屏左右分栏、窄屏单页自适应，浅色主题
- Windows：管理员提权、Wix 打包 MSI 安装包

## 后端 API

纯客户端项目，后端不在本仓库。后端地址**只**通过项目根目录 `.env` 配置（参考 `.env.example`），构建时写进应用，运行时零配置：

```
CLX_BASE_URL=https://你的域名
```

接口约定见 `docs/openapi.yaml`、`app-gpui/src/services/api.rs` / `app-tauri/src/api/index.ts`。

## 构建

前置依赖：Rust 稳定版、pnpm（Tauri 版需要）。构建前先在根目录创建 `.env`。

```bash
# Tauri 版
cd app-tauri
pnpm install
pnpm tauri dev      # 开发
pnpm tauri build    # 打包 MSI

# GPUI 版
cd app-gpui
cargo run           # 开发
cargo build --release
```

## 关于 GPUI

`app-gpui` 使用 Zed 的 GPUI 框架（Rust UI 库，暂无文档），当初为学习 GPUI 而写。

## 许可证

AGPL-3.0-only，见 [LICENSE](./LICENSE)。
