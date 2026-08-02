# 验证与开发检查

## 编译与测试

```bash
cargo fmt --check
cargo test --workspace --no-default-features
cargo test --workspace
cargo test --workspace --doc
```

默认 feature 包含 `watch`、`serve` 和 `search`；无默认 feature 的测试用于确认可选依赖和 feature gate 没有破坏基础构建。

## 构建现有文档

```bash
cargo run --no-default-features -F search -- build guide
```

这会使用当前源码构建仓库已有的用户指南。`doc/` 是架构说明 Markdown 目录，不是第二份 mdBook 工程，因此不应把它加入 `guide/book.toml` 的 SUMMARY。

## CLI 手工冒烟

```bash
cargo run -- init /tmp/mdbook-smoke
cargo run -- build /tmp/mdbook-smoke
cargo run -- test /tmp/mdbook-smoke
cargo run -- serve /tmp/mdbook-smoke
cargo run -- watch /tmp/mdbook-smoke
cargo run -- clean /tmp/mdbook-smoke
```

实际开发时可使用仓库中的示例书替代临时目录。调试构建流程时设置：

```bash
MDBOOK_LOG=mdbook_driver=debug cargo run -- build <book-dir>
```

## 相关测试位置

- `src/main.rs`：clap 命令结构断言。
- `src/cmd/watch/native.rs`、`src/cmd/watch/poller.rs`：监听和 gitignore 过滤测试。
- `tests/testsuite/`：构建、配置、Markdown、预处理器、渲染、搜索、主题和 CLI 集成测试。
- `tests/gui/runner.rs`：浏览器 GUI 测试。
- `crates/mdbook-compare`：跨版本 HTML 回归比较。

新增或修改 `src/`、`crates/` 的公共行为时，应同时更新本目录相关章节和用户向的 `guide/` 文档（如果用户可见）。
