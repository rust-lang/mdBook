# mdBook Go 重构进度跟踪

> 本文件是 M1～M6 执行的唯一权威状态来源。TaskCreate/TaskUpdate 反映当前会话的进度；本文件反映跨会话的累计状态。

## 基本信息

- 仓库根：`/Users/qhai-dev/qhai-dev/mdBook`
- Rust 基准：`src/` + `crates/` + `tests/testsuite/`
- Go 重构目录：`mdbook-go/`（与 Rust 并行，不替换 `src/`）
- 对照方式：保留 Rust 作为 oracle，Go 跑同一 fixture 后 diff 输出
- 工具链：Go 1.26.4 / Rust 1.96.1 / Cargo 1.96.1

## 全量任务分解

### M1：核心加载器 + 最小 build

- [x] M1.1 创建 `mdbook-go/` 目录与 `go.mod`
- [x] M1.2 实现 `internal/book/model.go`：`Book`、`BookItem`、`Chapter`、`Separator`、`PartTitle`、`SectionNumber`
- [x] M1.3 实现 `internal/config/config.go`：强类型 `Config`、`BookConfig`、`BuildConfig`、`RustConfig`、`HtmlConfig` 最小集
- [x] M1.4 实现 `internal/config/toml.go`：`book.toml` 解析 + 动态 `output.*` / `preprocessor.*` 字段
- [x] M1.5 实现 `internal/config/env.go`：`MDBOOK_*` 环境变量覆盖
- [x] M1.6 实现 `internal/summary/parser.go`：`SUMMARY.md` 解析（标题/前缀/编号/后置/嵌套/part title/separator/draft）
- [x] M1.7 实现 `internal/driver/loader.go`：从磁盘读取章节文件，组装 `Book` 树
- [x] M1.8 实现 `internal/driver/mdbook.go`：`MDBook` 结构 + `Load` + `Build` 最小版本
- [x] M1.9 实现 `internal/markdown/parser.go`：基于 `goldmark` 的最小 Markdown → HTML
- [x] M1.10 实现 `internal/html/renderer.go`：最小 HTML renderer（章节 HTML 写出）
- [x] M1.11 实现 `cmd/mdbook/main.go`：CLI 入口，至少 `build` 和 `init` 子命令
- [x] M1.12 实现 `internal/driver/init.go`：复制 `MDBook::init` 行为（创建 `book.toml`/`SUMMARY.md`/章节/gitignore）
- [x] M1.13 创建基础 fixture `mdbook-go/fixtures/basic/`
- [x] M1.14 实现 harness 脚本 `mdbook-go/harness/diff.sh`：分别跑 Rust 与 Go 后 diff
- [x] M1.15 跑通 baseline：Rust 输出与 Go 输出对 fixture 跑通 diff，差异符合 M1 已知范围
- [x] M1.16 写 `mdbook-go/README.md` 说明构建/运行/对照方式

### M2：HTML renderer

- [x] M2.1 加入 `goldmark` 扩展：表格、脚注、任务列表、删除线、定义列表
- [x] M2.2 实现标题 ID 生成与去重
- [x] M2.3 实现 admonition 转换
- [x] M2.4 实现 `.md` 链接到 `.html` 改写
- [x] M2.5 实现 TOC 生成
- [x] M2.6 实现多章节 HTML 输出
- [x] M2.7 实现 `index.html` 与首章复制
- [x] M2.8 实现非 Markdown 资源复制
- [x] M2.9 实现 404.html
- [x] M2.10 实现 print.html（单页打印版）
- [x] M2.11 实现 redirect
- [x] M2.12 实现静态资源 hash 与重写
- [x] M2.13 主题支持：默认主题内嵌 + 用户主题覆盖
- [ ] M2.14 fixture 覆盖：多级章节、表格、脚注、admonition、嵌套 SUMMARY
- [ ] M2.15 M2 验收：以上 fixture Rust/Go diff 为空

### M3：插件兼容

- [ ] M3.1 定义内部 `Preprocessor` / `Renderer` Go 接口
- [ ] M3.2 定义 `PreprocessorContext` / `RenderContext` 字段
- [ ] M3.3 实现 JSON 序列化对齐 Rust 端字段名
- [ ] M3.4 实现 `CmdPreprocessor`：stdin/stdout + `supports` 探测
- [ ] M3.5 实现 `CmdRenderer`：stdin JSON + 工作目录 + 退出码
- [ ] M3.6 内置 `links` 预处理器
- [ ] M3.7 内置 `index` 预处理器
- [ ] M3.8 预处理器排序：`before`/`after` 拓扑排序
- [ ] M3.9 `supports_renderer` 与 renderer 白名单
- [ ] M3.10 fixture：外部 preprocessor、renderer、复合插件链
- [ ] M3.11 M3 验收：与 Rust 端外部插件协议 diff 一致

### M4：CLI 完整化

- [ ] M4.1 子命令 `init` 完整化（theme 复制、gitignore）
- [ ] M4.2 子命令 `test`：调用 `rustdoc --test`
- [ ] M4.3 子命令 `clean`：删除构建目录并统计字节
- [ ] M4.4 子命令 `completions`：Bash/Zsh/Fish/PowerShell
- [ ] M4.5 全局参数：`--dir`、`--dest-dir`、`--open`
- [ ] M4.6 错误码与错误信息兼容
- [ ] M4.7 退出码 101 / backtrace 输出
- [ ] M4.8 fixture：CLI 调用行为与 Rust 一致
- [ ] M4.9 M4 验收：CLI 行为 diff 一致

### M5：开发体验

- [ ] M5.1 poll watcher：`walkdir` 扫描 + mtime/size 对比
- [ ] M5.2 native watcher：`fsnotify` + debounce
- [ ] M5.3 `.gitignore` 过滤与父目录处理
- [ ] M5.4 `extra_watch_dirs` 支持
- [ ] M5.5 `net/http` 静态文件服务
- [ ] M5.6 WebSocket live reload
- [x] M5.7 搜索索引：生成兼容的 `searchindex.js`（提前到 M2 完成，见下）
- [ ] M5.8 资源 hash、清理、复制
- [ ] M5.9 fixture：watch、serve、搜索
- [ ] M5.10 M5 验收：watch/serve 行为与 Rust 一致

### M6：并行回归与发布

- [ ] M6.1 大型 fixture 库（10+）
- [ ] M6.2 跨平台构建：linux/darwin/windows
- [ ] M6.3 性能基准：构建时间、内存、二进制大小
- [ ] M6.4 许可证报告：`go-licenses`
- [ ] M6.5 CI 工作流：Rust/Go 并行构建与对比
- [ ] M6.6 文档：迁移指南、限制、已知差异
- [ ] M6.7 发布：实验性 Go 二进制
- [ ] M6.8 回退方案：Rust 仍可作为默认
- [ ] M6.9 整体验收：所有 fixture diff 一致或仅允许已知差异

## 验证策略

每个里程碑在 `mdbook-go/harness/` 维护：

```text
harness/
├── run_rust.sh      # 跑 Rust mdbook build
├── run_go.sh        # 跑 Go mdbook-go build
├── diff.sh          # 跑双实现 + diff
└── fixtures/        # 共享 fixture（与 mdbook-go/fixtures/ 同步）
```

每次运行后 `diff` 输出必须为空，或者差异在 `KNOWN_DIFFS.md` 中显式列出。

## 进度记录

### 当前会话（2026-08-01 会话 2）

- 当前阶段：M2 实施中，M2.1～M2.13 完成，M2.14/M2.15 进行中
- **M2 关键成果：`fixtures/basic` 的 Rust 与 Go 输出已逐字节完全一致（43 个文件，`diff -rq` 无输出）**
  - 包含 `index.html`、各章节页、`toc.html`、`toc.js`、`print.html`、`404.html`、
    `searchindex.js`、`.nojekyll`，以及全部带 hash 的 CSS/JS/字体/图标资源
  - 资源指纹文件名（如 `css/general-e96d0476.css`、`book-609e4cb8.js`）与 Rust 完全相同
- 编译产物：`mdbook-go/bin/mdbook-go`

### M2 已落地的包

| 包 | 职责 | 对应 Rust |
|---|---|---|
| `internal/utils` | HTML 转义、`path_to_root`、slug、去重 ID、文件复制 | `mdbook-core/src/utils/{html,fs}.rs`、`mdbook-html/src/utils.rs` |
| `theme/`（`themedata`） | `go:embed` 内嵌默认前端资源 | `mdbook-html/front-end/` |
| `internal/theme` | 主题解析：内嵌默认 + 用户 `theme/` 覆盖 | `mdbook-html/src/theme/mod.rs` |
| `internal/hbs` | 自研 Handlebars 子集引擎（含 standalone 空白规则） | `handlebars` crate |
| `internal/fontawesome` | Font Awesome 图标 SVG | `font-awesome-as-a-crate` |
| `internal/html` | Markdown → 节点树 → 序列化，及 header anchor / 代码块 / admonition / 链接改写 | `mdbook-html/src/html/` |
| `internal/static` | 静态资源集合、SHA-256 指纹、`{{ resource }}` 重写 | `html_handlebars/static_files.rs` |
| `internal/search` | elasticlunr 索引 + Porter stemmer + 停用词 | `elasticlunr-rs` + `html_handlebars/search.rs` |
| `internal/render` | 渲染主流程、`make_data`、TOC helper、print、404、redirect | `html_handlebars/hbs_renderer.rs` |

### M2 期间关闭的 M1 遗留缺陷

- `internal/summary/parser.go` 重写：支持任意层级嵌套（原先仅 1 层），并按
  列表标记而非位置区分 prefix / numbered / suffix 章节。
- `internal/driver/loader.go`：章节编号改为层级编号（1、1.1、1.1.1），
  part title 不重置计数，draft 章节仍占用编号——均以 Rust 的 `toc.html` 为准校验。
- `book.SectionNumber.String()`：改为每段后带点（`1.1.`），与 Rust `Display` 一致；
  侧边栏靠点号数量推导缩进层级，此前会导致层级错误。
- `Chapter.HTMLPath()`：保留子目录结构（`guide/advanced/deep.html`），
  原实现的「展平为 `<parent>-<name>.html`」是错的。

### 提前完成的 M5 项

搜索索引（原计划 M5.7）在 M2 提前完成：章节页 `<head>` 中的
`window.path_to_searchindex_js` 引用带 hash 的索引文件名，不实现索引就无法做到
章节页逐字节一致。`internal/search` 的输出已与 Rust golden 逐字节相同。

### 依赖变更

新增 `golang.org/x/net`（`html` 分词器），用于解析 Markdown 内嵌的原生 HTML，
对应计划第 3 节中 `ego-tree`/`html5ever` → `golang.org/x/net/html` 的映射。

### 测试

| 测试 | 内容 | 状态 |
|---|---|---|
| `internal/hbs` | `index.hbs`/`toc.html.hbs` 渲染结果与 Rust 输出逐字节比对 | 通过 |
| `internal/fontawesome` | 图标 SVG 与 Rust 输出片段逐字节比对 | 通过 |
| `internal/search` | 索引 JSON 与 Rust golden 逐字节比对 | 通过 |
| `internal/html` | 以 `tests/testsuite/markdown/*/expected/*.html` 为 golden 回归 | 通过（2 项已知偏差跳过） |

### 已知偏差（goldmark 与 pulldown-cmark 的解析器差异）

登记在 `internal/html/markdown_golden_test.go` 的 `knownDeviations` 中：

1. `definition_lists/definition_lists`：goldmark 的定义列表要求「术语为单行纯文本」，
   含行内链接或跨行的术语不会变成 `<dt>`。
2. `basic_markdown/html`：开标签跨两行时，goldmark 视为 HTML 块，
   pulldown-cmark 则回退为段落内的行内 HTML。

两者都需要替换 goldmark 的块级解析才能消除，不影响当前 fixture。

### 下一步

1. M2.14：`fixtures/nested/` 已创建（四层嵌套、子目录章节、表格、脚注、
   admonition、任务列表、代码块、redirect、`additional-css`、fold、分隔符、
   前置/后置章节、draft）。当前有两个待处理项：
   - fixture 的 `[output.html.redirect]` 只给了带 `#` 片段的条目，Rust 同样报错，
     需要补一条不带片段的默认目标。
   - `git-repository-url` 场景下 `git_repository_icon` 默认值 `fa-github` 与
     图标类别推导的组合会查不到图标，需要对照 Rust 的实际取值修正。
2. M2.15：`harness/diff.sh` 改为严格逐字节比对，跑通 basic + nested。
3. 更新 `harness/KNOWN_DIFFS.md`：清空 M2 条目，仅保留上述解析器偏差。

### 会话历史

#### 2026-08-01 会话 1

- 完成 M1 全部任务。
- 工具链：Go 1.26.4 darwin/arm64，Rust 1.96.1。
- harness 跑通：`./harness/diff.sh basic --normalize` 报告预期差异。

#### 2026-08-01 会话 2

- 完成 M2.1～M2.13，`fixtures/basic` 双实现输出逐字节一致。
- 提前完成 M5.7 搜索索引。
- 关闭 4 项 M1 遗留缺陷。
