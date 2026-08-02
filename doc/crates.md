# `crates/` 核心职责

| Crate | 职责 | 关键实现 |
|---|---|---|
| `mdbook-core` | 共享 `Book`、`Config`、错误和文件系统工具 | `src/book.rs`, `src/config.rs`, `src/utils/` |
| `mdbook-driver` | 加载书籍、编排预处理器/渲染器、初始化和测试 | `src/mdbook.rs`, `src/load.rs`, `src/init.rs` |
| `mdbook-html` | 默认 HTML 后端、主题、搜索和静态资源 | `src/html_handlebars/`, `src/html/`, `src/theme/` |
| `mdbook-summary` | 解析 `SUMMARY.md` 为目录结构 | `src/lib.rs` |
| `mdbook-markdown` | 统一 pulldown-cmark 解析选项 | `src/lib.rs` |
| `mdbook-preprocessor` | 预处理器 trait、上下文和 JSON 输入协议 | `src/lib.rs` |
| `mdbook-renderer` | 渲染器 trait、上下文和 JSON 输入协议 | `src/lib.rs` |
| `mdbook-compare` | 比较两个 mdBook 版本的 HTML 输出 | `src/main.rs` |
| `xtask` | 测试、lint、文档、版本和 changelog 自动化 | `src/main.rs`, `src/changelog.rs` |

## `mdbook-core`

基础 crate 不依赖其他 mdBook crate。`Book` 是 `Vec<BookItem>` 组成的树，`BookItem` 包含 `Chapter`、`Separator` 和 `PartTitle`。章节同时保存真实 `source_path` 与渲染 `path`；index 预处理器可改变后者而不丢失前者。`Config` 负责 TOML、环境变量覆盖、点路径访问和插件动态配置。

## `mdbook-driver`

`MDBook` 聚合根目录、配置、书籍及 renderer/preprocessor 注册表。加载流程是读取 `src/SUMMARY.md`，调用 `mdbook-summary`，再递归读取章节文件生成 `Book`。构建时为每个 renderer 从原始书籍副本开始执行预处理器；预处理器按 `before`/`after` 约束拓扑排序。

内置预处理器包括：

- `links`：展开 include、rustdoc include、playground 和 title helper。
- `index`：将 README 的渲染路径转换为 index。
- `cmd`：调用外部预处理器。

内置 renderer 包括 Markdown renderer 和命令 renderer；未配置 HTML 之外的输出时默认使用 HTML。

## `mdbook-html`

`HtmlHandlebars` 实现 `Renderer`。它将 Markdown parser 事件转换为 `ego-tree`，再完成标题锚点、脚注、表格、admonition、代码隐藏、Rust playground 和链接改写。随后加载 Handlebars 主题，输出章节、TOC、`index.html`、404、打印页、redirect 和非 Markdown 资源。启用 `search` feature 时，还从 HTML tree 构建 elasticlunr 搜索索引，并对静态资源生成 hash 文件名。

## 扩展协议

`mdbook-preprocessor` 的 `Preprocessor` 接收 `PreprocessorContext + Book` 并返回新的 `Book`；`mdbook-renderer` 的 `Renderer` 接收 `RenderContext` 并写出目标产物。进程内插件实现 trait，进程外插件通过 stdin/stdout 传输 JSON，driver 中的 `CmdPreprocessor` 和 `CmdRenderer` 是协议实现。

## 开发工具

`mdbook-compare` 构建两份书籍，使用 `tidy` 规范化并比较 `<main>` 内容，用于回归检查。`xtask` 集中执行 workspace 测试、clippy、rustdoc、fmt、semver、GUI、ESLint、版本 bump 和 changelog 生成。
