# codex-mcp-rs

[English](README.md)

[![CI](https://github.com/missdeer/codex-mcp-rs/workflows/CI/badge.svg)](https://github.com/missdeer/codex-mcp-rs/actions)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Rust Version](https://img.shields.io/badge/rust-1.77.2%2B-blue.svg)](https://www.rust-lang.org)
[![MCP Compatible](https://img.shields.io/badge/MCP-Compatible-green.svg)](https://modelcontextprotocol.io)

高性能的 Rust 实现的 MCP（模型上下文协议）服务器，封装了 Codex CLI 以支持 AI 辅助编程任务。

## 功能特性

- **MCP 协议支持**：使用 Rust SDK 实现官方模型上下文协议
- **Codex 集成**：封装 Codex CLI，通过 MCP 实现 AI 辅助编程
- **会话管理**：通过会话 ID 支持多轮对话
- **沙箱安全**：可配置的沙箱策略（只读、工作区写入、完全访问）
- **图像支持**：可将图像附加到提示中以提供视觉上下文
- **异步运行时**：基于 Tokio 构建，实现高效的异步 I/O
- **跨平台**：为 Linux、macOS 和 Windows（x64 和 arm64）提供预编译二进制文件

## 支持的平台

| 平台 | 架构 | 二进制文件 |
|------|------|-----------|
| Linux | x86_64 | `codex-mcp-rs_Linux_x86_64.tar.gz` |
| Linux | arm64 | `codex-mcp-rs_Linux_arm64.tar.gz` |
| macOS | Universal (x64 + arm64) | `codex-mcp-rs_Darwin_universal.tar.gz` |
| Windows | x86_64 | `codex-mcp-rs_Windows_x86_64.zip` |
| Windows | arm64 | `codex-mcp-rs_Windows_arm64.zip` |

## 前置要求

- Rust 1.77.2+（Windows 命令行参数转义修复所需）
- 已安装并配置 [Codex CLI](https://github.com/anthropics/codex)
- Claude Code 或其他 MCP 客户端

## 构建

```bash
# 调试构建
cargo build

# 发布构建
cargo build --release
```

## 运行

服务器通过 stdio 传输进行通信：

```bash
cargo run
```

或在构建后：

```bash
./target/release/codex-mcp-rs
```

## 快速开始

最快的入门方式是使用 npx：

```bash
npx @missdeer/codex-mcp-rs
```

此命令会自动安装适合您平台（Windows/macOS/Linux，x64/arm64）的预编译二进制文件并启动 MCP 服务器。无需手动安装。

添加到 Claude Code：

```bash
claude mcp add codex-rs -s user --transport stdio -- npx @missdeer/codex-mcp-rs
```

## 安装

### 方式一：通过 npx 使用（推荐）

使用 npx 是最简单的方式 - 它会自动处理二进制文件安装：

```bash
npx @missdeer/codex-mcp-rs
```

**执行过程：**
1. npm 安装特定平台的二进制包（`@codex-mcp-rs/darwin-universal`、`@codex-mcp-rs/linux-x64` 等）
2. 在 stdio 传输上启动 MCP 服务器

**添加到 Claude Code MCP 配置：**

```bash
claude mcp add codex-rs -s user --transport stdio -- npx @missdeer/codex-mcp-rs
```

**或者全局安装以加快启动速度：**

```bash
npm install -g @missdeer/codex-mcp-rs
claude mcp add codex-rs -s user --transport stdio -- codex-mcp-rs
```

全局安装会在本地缓存二进制文件，实现即时启动。

### 方式二：通过安装脚本安装（Linux/macOS）

自动下载并安装最新版本到 `/opt/codex-mcp-rs/`：

```bash
curl -sSL https://raw.githubusercontent.com/missdeer/codex-mcp-rs/master/scripts/install.sh | bash
```

此脚本将：
- 检测您的平台和架构
- 从 GitHub 下载最新版本
- 将二进制文件安装到 `/opt/codex-mcp-rs/codex-mcp-rs`
- 自动添加到您的 Claude Code MCP 配置

### 方式三：从发布版安装

从[发布页面](https://github.com/missdeer/codex-mcp-rs/releases)下载适合您平台的二进制文件，解压后添加到 MCP 配置：

```bash
claude mcp add codex-rs -s user --transport stdio -- /path/to/codex-mcp-rs
```

### 方式四：从源码构建

```bash
git clone https://github.com/missdeer/codex-mcp-rs.git
cd codex-mcp-rs
cargo build --release
claude mcp add codex-rs -s user --transport stdio -- $(pwd)/target/release/codex-mcp-rs
```

## 工具使用

服务器提供一个 `codex` 工具，具有以下参数：

### 必需参数

- `PROMPT`（字符串）：给 Codex 的任务指令
- `cd`（字符串）：工作目录路径

### 可选参数

- `sandbox`（字符串）：沙箱策略 - `"read-only"`（默认）、`"workspace-write"` 或 `"danger-full-access"`
- `SESSION_ID`（字符串）：恢复之前的会话以进行多轮对话
- `skip_git_repo_check`（布尔值）：允许在 git 仓库外运行（默认：`false`）
- `return_all_messages`（布尔值）：返回完整的推理过程（默认：`false`）
- `image`（数组）：要附加的图像文件路径
- `model`（字符串）：覆盖 Codex 模型
- `yolo`（布尔值）：禁用所有提示和沙箱
- `profile`（字符串）：从 `~/.codex/config.toml` 加载配置文件

## 测试

项目具有全面的测试覆盖：

```bash
# 运行所有测试
cargo test

# 运行覆盖率测试
cargo tarpaulin --out Html

# 查看详细测试指南
cat TESTING.md
```

测试类别：
- **单元测试**（10 个）：核心功能（escape_prompt、Options）
- **集成测试**（10 个）：端到端场景
- **服务器测试**（5 个）：MCP 协议实现
- **CI 测试**：多平台验证

总计：25 个测试通过 ✅

当前测试覆盖率：参见 [Codecov](https://codecov.io/gh/missdeer/codex-mcp-rs)

## 架构

详细架构文档请参见 [CLAUDE.md](./CLAUDE.md)。

## 与其他实现的比较

| 特性 | codex-mcp-rs (Rust) | codexmcp (Python) | codex-mcp-go |
|------|---------------------|-------------------|--------------|
| 语言 | Rust | Python | Go |
| 性能 | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ |
| 内存占用 | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ |
| 二进制大小 | 中等 | N/A | 小 |
| 启动时间 | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| 会话管理 | ✓ | ✓ | ✓ |
| 图像支持 | ✓ | ✓ | ✓ |
| 沙箱策略 | ✓ | ✓ | ✓ |

## 相关项目

- [codex-mcp-go](https://github.com/w31r4/codex-mcp-go) - Go 实现
- [geminimcp](https://github.com/GuDaStudio/geminimcp) - Gemini CLI 的 Python MCP 服务器

## 贡献

欢迎贡献！请参阅 [CONTRIBUTING.md](./CONTRIBUTING.md) 了解指南。

## 许可证

本项目采用双重许可：

### 非商业/个人使用 - GNU 通用公共许可证 v3.0

可免费用于个人项目、教育目的、开源项目和非商业用途。完整的 GPLv3 许可证文本请参见 [LICENSE](LICENSE)。

### 商业/工作场所使用 - 需要商业许可证

**如果您在商业环境、工作场所或用于任何商业目的使用 codex-mcp-rs，您必须获得商业许可证。**

这包括但不限于：
- 在工作中使用本软件（任何组织）
- 集成到商业产品或服务中
- 用于客户工作或咨询
- 作为 SaaS/云服务的一部分提供

**联系方式**：missdeer@gmail.com 获取商业许可咨询。

更多详情请参见 [LICENSE-COMMERCIAL](LICENSE-COMMERCIAL)。

## Star 历史

[![Star History Chart](https://starchart.cc/missdeer/codex-mcp-rs.svg)](https://starchart.cc/missdeer/codex-mcp-rs)
