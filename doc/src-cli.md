# `src/` CLI 与命令层

`src/` 不直接实现 Markdown 解析或 HTML 生成，而是负责参数解析、运行时配置和调用 `mdbook-driver::MDBook`。

## 入口：`src/main.rs`

1. 初始化 `tracing_subscriber` 日志，读取 `MDBOOK_LOG`。
2. 通过 clap 构造命令树，并注册 `init`、`build`、`test`、`clean`、`completions`，以及 feature 控制的 `watch`、`serve`。
3. 将 `ArgMatches` 分派到 `src/cmd/*::execute`。
4. 统一记录错误和 backtrace，错误退出码为 101。

## 公共参数：`src/cmd/command_prelude.rs`

`CommandExt` 为命令复用以下参数：

- `--dir`：书根目录
- `--dest-dir`：构建输出目录
- `--open`：构建后打开入口页
- `--watcher {poll,native}`：监听实现

`set_dest_dir` 将命令行输出目录写入 `book.config.build.build_dir`，保证所有命令使用同一配置入口。

## 子命令职责

| 命令 | 主要流程 |
|---|---|
| `build` | `MDBook::load` → 设置输出目录 → `book.build()` → 可选打开 `index.html` |
| `init` | `MDBook::init`/`BookBuilder` 创建 `book.toml`、`SUMMARY.md`、章节、主题和 gitignore |
| `test` | 加载书籍，调用 `MDBook::test` 或 `test_chapter`，对预处理后的章节运行 `rustdoc --test` |
| `clean` | 加载配置，遍历并删除构建目录，同时统计释放空间 |
| `watch` | 选择 native 或 poller watcher，文件变更后重新加载并构建 |
| `serve` | 首次构建后启动 axum HTTP 服务，通过 WebSocket 推送 live reload；watcher 负责重建 |

## 文件监听

- `watch/native.rs` 使用 `notify` 和 debounce，监听源目录、主题、`book.toml` 及额外目录。
- `watch/poller.rs` 通过 `walkdir` 扫描文件类型、mtime 和大小，比较快照发现变化。
- 两种实现都应用 `.gitignore` 过滤，并支持 `extra_watch_dirs`。

因此 CLI 层的关键原则是：命令模块保持薄，书籍生命周期和构建细节统一委托给 `MDBook`。
