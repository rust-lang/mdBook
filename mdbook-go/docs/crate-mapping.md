# mdbook-go internal 包 ↔ Rust crates 映射

> 依据：Go 源码注释中自带的 "port of / mirrors / crates/..." 引用。
> 更新时间：2026-08-09，分支 v1。
>
> Rust 侧结构：`crates/` 下 9 个 crate，外加 workspace 根二进制 crate `mdbook`
>（`src/main.rs` + `src/cmd/`，即 CLI 命令层）。

## 1. 总览表

| Go internal 包 | Rust 对应（crate::module） | 对应源文件 | 对齐度 |
|---|---|---|---|
| `book` | `mdbook-core::book` | `book.rs` | ✅ 1:1 |
| `config` | `mdbook-core::config` | `config.rs`, `html.rs` | ⚠️ 键语义 1:1，文件格式偏离：Go 侧 `doclens.yaml`（YAML），Rust 侧 `book.toml`（TOML），无 `MDBOOK_*` env 覆盖；默认值也偏离：Go 侧 `src: docs` + `build-dir: .doclens`，Rust 侧默认 `src` + `book`（见 tests/README.md） |
| `utils` | `mdbook-core::utils` + `mdbook-html::utils` | `fs.rs`, `html.rs`, `utils.rs` | ✅ 1:1（跨两个 crate 的同名模块） |
| `summary` | `mdbook-summary` | `lib.rs` | ❌ 已删（2026-08-09）：SUMMARY.md 被 `doclens.yaml` 的 `[chapters]` 段取代，解析逻辑并入 `runner/loader.go`（`loadChapters` / `newChapter`），`mdbook-summary` 是 Go 端唯一无对应包的 crate |
| `serve`（已并入 pkg/cmd，2026-08-09） | `mdbook` bin: `src/cmd/serve.rs` | `serve.rs` | ✅ 1:1（现位于 `pkg/cmd/serve/serve.go`，命令+服务器+reload 单文件，同 Rust） |
| `assets`（原 `theme`，2026-08-09 迁入 internal） | `mdbook-html::theme` | `theme/mod.rs` | ✅ 1:1（现位于 `internal/static`，go:embed 前端资产，无模板字段） |
| `static` | `mdbook-html::html_handlebars` | `static_files.rs` | ✅ 1:1 |
| `search` | `mdbook-html::html_handlebars` | `search.rs`（elasticlunr） | ✅ 1:1 |
| `cli` | `mdbook` bin: `main.rs` | `log_backtrace` / HandleError；2026-08-09 并入 `cmd/doclens/main.go`（formatError + exit 101） | ✅ 1:1 |
| `runner` | `mdbook-driver` + `mdbook` bin: `src/cmd/*` + `src/cmd/watch/*` | 见下表 | ✅ 1:1（仅 markdown renderer 与 test 未移植） |
| `html` | `mdbook-html::html` 模块 | `tree.rs`, `serialize.rs`, `admonitions.rs`, `hide_lines.rs`, `mod.rs` | ⚠️ 名字误导（见 §3） |
| `render` | `mdbook-html::html_handlebars` | `hbs_renderer.rs`（+ 部分 `search.rs`） | ⚠️ 名字误导（见 §3） |
| `plugin` | `mdbook-preprocessor` + `mdbook-renderer` | 见下表 | ✅ 1:1（2026-08-09 builtin 实现迁出至 runner） |
| `tplgotpl` | —（无 Rust 对应） | Go 专用 html/template 引擎，替代 handlebars | ❓ Go 专用 |
| `watch` | `mdbook` bin: `src/cmd/watch` | `pkg/cmd/watch/`：命令壳（watch.go）+ 引擎（engine.go / watch_native.go / watch_poll*.go / gitignore.go），2026-08-09 由 `internal/driver`（现 `internal/runner`）的 `watch*.go` 迁入 | ✅ |

## 2. 混杂包的内部映射

### `internal/runner`（9 个文件 = 3 处来源）

| Go 文件 | Rust 对应 |
|---|---|
| `build.go` | `mdbook-driver::mdbook.rs::MDBook::build` |
| `loader.go` | `mdbook-driver` 加载流程（load） |
| `init.go` | `mdbook` bin: `src/cmd/init.rs` |
| `cmd.go` | `mdbook-driver::builtin_preprocessors::cmd.rs` |
| `index.go` | `mdbook-driver::builtin_preprocessors::index.rs` |
| `links.go` | `mdbook-driver::builtin_preprocessors::links.rs` |
| `registry.go` | `mdbook-driver::mdbook.rs::determine_preprocessors` / `preprocessor_should_run` |
| `links_test.go` / `registry_test.go` | 对应 Rust 单元测试 |

> 2026-08-09 起，原本混在 `driver` 里的 3 个 `src/cmd` 命令实现已迁出：
> `clean.go` → `pkg/cmd/clean.go`，`open.go` → `pkg/cmd/open.go`，`test.go` 删除
> （Go 未移植 test 命令）。builtin preprocessor 实现（cmd/index/links）则于
> 同日从 `plugin` 迁回，与编排同包（对齐 Rust 的 crate 边界）。

### `internal/plugin`（2 个文件 = 2 处来源）

| Go 文件 | Rust 对应 |
|---|---|
| `plugin.go`（Preprocessor 接口） | `mdbook-preprocessor::lib.rs::Preprocessor` |
| `plugin.go`（Renderer 接口） | `mdbook-renderer::lib.rs::Renderer` |
| `protocol.go` | wire 格式：`mdbook-preprocessor`（PreprocessorContext）、`mdbook-renderer`（RenderContext）、`mdbook-core`（Book/Chapter/Config 的 serde 序列化） |

> 2026-08-09 起 `plugin` 只保留契约层（接口 + wire 协议）；
> builtin 实现（cmd/index/links + registry）已迁回 `internal/runner`。

## 3. 命名错位（最容易被看反的两处）

- **`internal/html` ≠ HTML 渲染**。它的内容全部来自 `mdbook-html::html` 模块：
  goldmark markdown → HTML 事件树（`tree.rs`）、序列化（`serialize.rs`）、admonitions、hide_lines 等——本质是 **markdown 渲染管线**。
- **`internal/render` 才是 HTML 渲染器**。`render/render.go` 明确写 "port of
  crates/mdbook-html/src/html_handlebars/hbs_renderer.rs"，还依赖 `html`、`search`、`static`、`theme`、`tplgotpl` 五个包。

## 4. Rust 侧无 Go 对应的部分

| Rust | 说明 |
|---|---|
| `crates/mdbook-markdown` | 只是 pulldown_cmark 的薄封装；Go 直接用 goldmark 依赖，无包 |
| `crates/mdbook-compare` | 对比工具，Go 无 |
| `crates/xtask` | 构建任务，Go 无（对应 `Makefile`/`tailwindcss.sh`） |

## 5. 乱点小结（供重构决策参考）

1. **一个 crate 被拆成 6 个兄弟包**：`mdbook-html` = `html` + `render` + `theme` + `static` + `search` + `tplgotpl`，且相互 import，crate 级封装关系不可见。
2. **名字与内容错位**：`html`（markdown 管线）与 `render`（HTML 渲染器）反直觉；`tplgotpl` 不可读。
3. ~~**`plugin` 跨 4 个 crate 的来源**，`driver` 混入 3 个 `src/cmd` 命令的实现。~~（已解决 2026-08-09：builtin 实现迁回 `runner`，`plugin` 只剩接口 + wire；`clean`/`open` 迁至 `pkg/cmd`、`test` 删除，`runner` 不再混入 `src/cmd` 命令实现）
4. ~~`internal/watch/` 空目录~~（已删除 2026-08-09），watch 实现先散在 `driver/watch*.go`，2026-08-09 随引擎一并迁入 `pkg/cmd/watch/`。
5. **`tplgotpl`** 是 Go 专用新增物，没有 Rust 锚点——重构时需要单独决定归属（它是 handlebars 的替代品，语义上属 `mdbook-html::html_handlebars`）。

## 附：CLI 命令层对照（pkg/cmd ↔ src/cmd）

| Go `pkg/cmd` | Rust `src/cmd` |
|---|---|
| build | build |
| clean | clean |
| create | init |
| completion | completions |
| serve | serve |
| watch | watch |
| version | `main.rs`（--version） |
| — | test（Go 未移植，2026-08-09 随 test.go 一并删除） |
