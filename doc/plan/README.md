# mdBook Go 重构计划

> 状态：方案评审中
>
> 本文只描述计划，不代表已经开始 Go 代码实现。目标是先保持 mdBook 的用户行为和插件协议，再逐步替换内部实现。

## 1. 背景与目标

当前 mdBook 使用 Rust workspace 构建，核心由 `src/` CLI、`mdbook-driver` 编排层、`mdbook-core` 数据模型、Markdown/HTML 处理模块以及 renderer/preprocessor 插件协议组成。

Go 重构的目标不是逐文件翻译 Rust，而是逐步实现一个行为兼容的 Go 版本，重点保证：

1. `book.toml` 配置兼容。
2. `SUMMARY.md` 解析兼容。
3. Markdown 和 HTML 输出行为兼容。
4. `build`、`init`、`test`、`clean`、`watch`、`serve` 命令兼容。
5. preprocessor 和 renderer 的外部 JSON 协议兼容。
6. 现有书籍、主题、插件和构建流程可以继续使用。

建议分三个兼容等级：

- **Level 1：功能兼容**：可以加载书籍并生成基本 HTML。
- **Level 2：行为兼容**：路径、配置、链接、标题 ID、插件行为一致。
- **Level 3：输出兼容**：HTML、CSS、JS、搜索索引和打印页尽量一致。

推荐先达到 Level 1，再推进 Level 2，最后根据实际需求评估 Level 3。

## 2. 推荐项目结构

```text
mdbook-go/
├── go.mod
├── cmd/
│   └── mdbook/
│       └── main.go
├── internal/
│   ├── book/
│   │   ├── model.go
│   │   └── tree.go
│   ├── config/
│   │   ├── config.go
│   │   ├── toml.go
│   │   └── env.go
│   ├── summary/
│   │   ├── parser.go
│   │   └── model.go
│   ├── markdown/
│   │   ├── parser.go
│   │   └── options.go
│   ├── driver/
│   │   ├── mdbook.go
│   │   ├── loader.go
│   │   ├── builder.go
│   │   ├── preprocessor.go
│   │   └── renderer.go
│   ├── html/
│   │   ├── renderer.go
│   │   ├── tree.go
│   │   ├── template.go
│   │   ├── theme.go
│   │   ├── search.go
│   │   └── static.go
│   ├── plugin/
│   │   ├── preprocessor.go
│   │   ├── renderer.go
│   │   └── protocol.go
│   └── watch/
│       ├── native.go
│       └── poller.go
├── pkg/
│   ├── preprocessor/
│   └── renderer/
├── templates/
├── theme/
└── tests/
```

第一阶段建议把核心代码放在 `internal/`，不要过早公开 Go API。等数据模型、协议和行为稳定后，再将适合第三方使用的接口移动到 `pkg/`。

## 3. Rust 依赖到 Go 的映射

| Rust 依赖/模块 | Go 方案 | 说明 |
|---|---|---|
| `clap` | `cobra` + `pflag` | 子命令和参数解析 |
| `clap_complete` | Cobra 补全能力 | 生成 Bash/Zsh/Fish/PowerShell 补全 |
| `serde`/`serde_json` | `encoding/json` | JSON 协议和模型序列化 |
| `toml` | `pelletier/go-toml/v2` | `book.toml` 解析，支持动态配置 |
| `pulldown-cmark` | `yuin/goldmark` | Markdown AST 和扩展 |
| `ego-tree`/`html5ever` | `golang.org/x/net/html` | HTML DOM、tokenizer 和序列化 |
| `handlebars` | Handlebars 兼容库或 `html/template` | 若要求旧主题兼容，应优先兼容 Handlebars |
| `elasticlunr-rs` | 先生成兼容的 `searchindex.js` | 不建议第一阶段替换搜索数据格式 |
| `notify` | `fsnotify` | 文件系统事件监听 |
| `ignore` | gitignore matcher 或自定义封装 | 处理 `.gitignore` 和额外监听目录 |
| `walkdir` | `filepath.WalkDir` | 递归目录遍历 |
| `axum`/`tokio` | `net/http` | 静态文件服务 |
| WebSocket | `nhooyr.io/websocket` 或 `gorilla/websocket` | live reload |
| `anyhow`/`tracing` | `fmt.Errorf` + `errors` + `log/slog` | 错误包装和结构化日志 |
| `sha2`/`hex` | `crypto/sha256` + `encoding/hex` | 静态资源 hash |
| `tempfile` | `os.MkdirTemp` | 测试临时目录 |

## 4. 分阶段实施路线

### 阶段 0：定义兼容目标

在开始编码前确定：

- 是否必须兼容现有第三方主题。
- 是否必须兼容现有 preprocessor/renderer。
- 是否要求 HTML 字节级一致，还是只要求浏览器渲染一致。
- 是否保留 Rust 版本作为 fallback。
- Go 版本是独立仓库，还是暂时作为当前仓库的子目录。

推荐默认目标：

```text
配置、目录结构、插件协议和主要 CLI 行为兼容；HTML 先保证语义和视觉一致，不追求字节级一致。
```

### 阶段 1：建立 Rust 基准测试

从 Rust mdBook 生成 golden fixtures：

```text
fixtures/
├── basic/
├── nested-summary/
├── markdown-features/
├── includes/
├── preprocessors/
├── themes/
├── redirects/
├── search/
└── watch/
```

每个 fixture 保存：

- `book.toml`
- `SUMMARY.md`
- Markdown 源文件
- 主题和静态资源
- 预期输出
- 正常和错误场景

重点覆盖：

- 多级章节和 draft chapter
- `README.md` 与 `index.md`
- 中文路径和特殊字符
- 图片、链接、表格、脚注、admonition
- include、rustdoc include、playground
- 搜索、自定义主题、redirect
- 外部 preprocessor 和 renderer

Rust 版本作为行为基准，Go 版本对同一 fixture 运行后进行目录、HTML 结构和日志差异比较。

### 阶段 2：实现核心数据模型

对应 Rust：

```text
crates/mdbook-core/src/book.rs
crates/mdbook-core/src/config.rs
```

首先实现：

```go
type Book struct {
    Items []BookItem `json:"items"`
}

type Chapter struct {
    Name        string
    Content     string
    Number      []int
    Path        string
    SourcePath  string
    ParentNames []string
    SubChapters []Chapter
}

type Config struct {
    Book         BookConfig
    Build        BuildConfig
    Rust         RustConfig
    Output       map[string]any
    Preprocessor map[string]any
}
```

必须优先验证：

- `source_path` 与渲染 `path` 分离。
- draft chapter 不需要实际文件。
- 章节编号和父章节名称正确。
- 配置动态字段可以透传给插件。
- 环境变量覆盖规则兼容。
- JSON 字段名和结构兼容现有协议。

### 阶段 3：实现配置和 SUMMARY loader

执行顺序：

```text
Config loader
    ↓
SUMMARY parser
    ↓
Book loader
    ↓
MDBook
```

`SUMMARY.md` 解析应基于 Markdown AST 或事件流，不建议使用正则直接解析。

优先支持：

1. 普通章节链接。
2. 嵌套章节。
3. Part Title。
4. Separator。
5. 前置、编号和后置章节。
6. draft chapter。
7. 重复文件检查和行列号错误。

### 阶段 4：实现最小 Markdown/HTML renderer

第一阶段只实现：

- 标题
- 段落
- 粗体、斜体
- 链接和图片
- 列表
- 引用
- 代码块
- 表格

随后补充：

1. 脚注。
2. 任务列表和删除线。
3. 定义列表。
4. admonition。
5. 标题属性和稳定 ID。
6. 原生 HTML。
7. Rust 代码隐藏行。
8. Playground。
9. Font Awesome。
10. TOC、打印页、404 和 redirect。

Go 内部接口建议：

```go
type Renderer interface {
    Name() string
    Render(ctx *RenderContext) error
}

type HTMLRenderer struct {
    Theme Theme
}
```

如果要兼容现有主题，应优先选择 Handlebars 兼容实现；如果允许 Go 版本使用新主题，可以先使用标准库 `html/template`。

### 阶段 5：实现 preprocessor/renderer 协议

内部接口：

```go
type Preprocessor interface {
    Name() string
    Run(ctx *PreprocessorContext, book *Book) (*Book, error)
    SupportsRenderer(renderer string) bool
}

type Renderer interface {
    Name() string
    Render(ctx *RenderContext) error
}
```

外部 preprocessor：

```text
stdin  = (PreprocessorContext, Book) JSON
stdout = processed Book JSON
```

外部 renderer：

```text
stdin  = RenderContext JSON
cwd    = destination directory
status = process exit code
```

必须保持兼容：

- JSON 字段名。
- stdin/stdout 约定。
- supports 探测行为。
- 进程退出码。
- 工作目录。
- stderr 转发。
- 自定义 command 配置。
- renderer/preprocessor 排序规则。

### 阶段 6：实现 CLI

CLI 层只负责参数、日志、路径和 driver 调用，不应把构建逻辑写进命令处理函数。

建议命令结构：

```text
cmd/mdbook/main.go
internal/cli/root.go
internal/cli/init.go
internal/cli/build.go
internal/cli/test.go
internal/cli/clean.go
internal/cli/watch.go
internal/cli/serve.go
```

命令实施顺序：

```text
build → init → clean → test → completions → watch → serve
```

### 阶段 7：实现 watch 和 serve

先实现 poll watcher：

```text
filepath.WalkDir
    ↓
文件状态快照
    ↓
mtime/size/type 对比
    ↓
触发重建
```

再实现 `fsnotify` native watcher，并补充：

- `.gitignore` 过滤。
- 父目录规则处理。
- `extra_watch_dirs`。
- theme 和 `book.toml` 监听。
- 防抖。

`serve` 先实现 `net/http` 静态文件服务，再加入：

- WebSocket live reload。
- 构建错误处理。
- 404。
- `--open`。
- 端口冲突处理。

### 阶段 8：补齐测试和工具链

`mdbook test` 应继续调用系统 `rustdoc --test`，不建议在 Go 项目中重写 Rust 编译器逻辑。

补充：

- 单元测试。
- 集成测试。
- 快照测试。
- 跨平台测试。
- GUI 测试。
- 性能和内存测试。
- 许可证检查。

## 5. 里程碑与验收标准

### M1：核心加载器

完成：

- `book.toml`
- `SUMMARY.md`
- 章节读取
- `Book`
- `Config`
- 最小 `build`

验收：能够对基础 fixture 生成 HTML。

### M2：HTML renderer

完成：

- goldmark Markdown 解析。
- 多章节 HTML 输出。
- TOC。
- 静态资源复制。
- 基础主题支持。

### M3：插件兼容

完成：

- Preprocessor 接口。
- Renderer 接口。
- 外部 JSON 协议。
- `links` 和 `index` 内置预处理器。
- 自定义 command。

### M4：CLI 完整化

完成：

- init。
- clean。
- test。
- completions。
- 参数和错误码兼容。

### M5：开发体验

完成：

- watch。
- serve。
- WebSocket live reload。
- 搜索索引。
- 资源 hash。
- 打印页和 redirect。

### M6：并行回归与发布

完成：

- Rust/Go 双实现输出对比。
- 跨平台构建。
- 性能测试。
- 许可证清单。
- 文档迁移。
- Go 版本实验性发布。

## 6. 迁移策略

不建议一次性删除 Rust 实现，推荐双实现并行：

```text
Rust mdBook：行为基准和 fallback
Go mdBook：逐模块实现和兼容性验证
```

可以增加临时比较工具：

```bash
mdbook-compare \
  --rust ./target/debug/mdbook \
  --go ./bin/mdbook-go \
  --book ./fixtures/basic
```

推荐切换步骤：

1. Go 版本作为实验性二进制发布。
2. 在 CI 中与 Rust 版本并行构建和测试。
3. 先覆盖基础构建和插件协议。
4. 完成主要兼容性后再替换默认实现。
5. 保留 Rust 版本一段时间作为回退方案。

## 7. 主要技术风险

### Markdown 行为差异

`pulldown-cmark` 和 `goldmark` 的 AST、转义、脚注、HTML 边界行为可能不同。必须使用真实 mdBook fixture 做差异测试。

### Handlebars 主题兼容

Go `html/template` 不能直接解析 Handlebars。若必须兼容现有第三方主题，需要采用 Handlebars 兼容实现，并保留 helper、变量和 partial 语义。

### 插件协议兼容

插件协议是重构中最不能随意修改的边界。需要为 `PreprocessorContext`、`RenderContext` 和 `Book` JSON 建立 golden fixtures。

### HTML 输出差异

如果需要 DOM 或字节级兼容，需要继续保留现有主题、CSS、JS、搜索前端和资源命名逻辑，工作量会明显增加。

### Rust 代码测试

`mdbook test` 依赖 `rustdoc`。Go 版本应继续调用系统 Rust 工具链，而不是尝试替代 Rust 编译器。

## 8. 推荐的第一步

建议先实现一个独立的 Go MVP，仅包含：

```text
book.toml
SUMMARY.md
Markdown 章节
HTML 输出
build 命令
```

完成后用 3～5 个基础 fixture 与 Rust 版本比较，再决定是否投入插件协议、watch、serve、搜索和完整主题兼容工作。

## 9. 许可证注意事项

Rust workspace 使用 MPL-2.0。第三方依赖主要使用 MIT、Apache-2.0、MIT/Apache-2.0 双许可证，也包含 Font Awesome 相关的 CC-BY-4.0 和 OFL 资源。

Go 重构发布前应生成依赖许可证报告，例如：

```bash
go-licenses csv ./...
go-licenses report ./...
```

继续使用 Font Awesome 时，应保留图标署名、字体许可证和相关版权声明。
