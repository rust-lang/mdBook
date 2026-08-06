# mdBook Go 重构进度跟踪

> 本文件是 M1～M6 执行的唯一权威状态来源。TaskCreate/TaskUpdate 反映当前会话的进度；本文件反映跨会话的累计状态。

## 基本信息

- 仓库根：`/Users/qhai-dev/qhai-dev/mdBook`
- Rust 基准：`src/` + `crates/` + `tests/testsuite/`
- Go 重构目录：`mdbook-go/`（与 Rust 并行，不替换 `src/`）
- 对照方式：保留 Rust 作为 oracle，Go 跑同一 fixture 后 diff 输出
- 工具链：Go 1.26.4 / Rust 1.96.1 / Cargo 1.96.1
- 上一会话环境：macOS arm64（Go 与 Rust 工具链可用）；2026-08-04 会话 7 切到
  Windows Git Bash，工具链已就地安装为 Go 1.26.4 / Rust 1.96.0，可完整运行 harness；
  当前会话回到 macOS arm64（Go 1.26.4 / Rust 1.96.1）。

## 全量任务分解

> ⚠️ **回归基线与删除 rust 前置条件**：见 [`testing.md`](testing.md)（2026-08-05 v0.2 后第一轮扫描：25 PASS / 5 DIFF / 6 SKIP / 12 BUILD_FAIL；删除 rust 前必须修完 5 处 silent validation gap + 3 处 functional gap）。

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
  - ⚠️ 实际落地位置：`internal/html/builder.go`（goldmark 直接在 HTML builder 内集成），未单独建 `internal/markdown` 包
- [x] M1.10 实现 `internal/html/renderer.go`：最小 HTML renderer（章节 HTML 写出）
  - ⚠️ 实际落地：`internal/render/render.go`（主流程） + `internal/html/*`（节点树、序列化、admonition、链接改写）
- [x] M1.11 实现 `cmd/mdbook/main.go`：CLI 入口，至少 `build` 和 `init` 子命令
- [x] M1.12 实现 `internal/driver/init.go`：复制 `MDBook::init` 行为（创建 `book.toml`/`SUMMARY.md`/章节/gitignore）
- [x] M1.13 创建基础 fixture `mdbook-go/fixtures/basic/`
- [x] M1.14 实现 harness 脚本 `mdbook-go/harness/diff.sh`：分别跑 Rust 与 Go 后 diff
- [x] M1.15 跑通 baseline：Rust 输出与 Go 输出对 fixture 跑通 diff，差异符合 M1 已知范围
- [x] M1.16 写 `mdbook-go/README.md` 说明构建/运行/对照方式
  - ⚠️ README 仍停留在 M1 视角，需要更新到当前 M2 阶段

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
- [x] M2.14 fixture 覆盖：多级章节、表格、脚注、admonition、嵌套 SUMMARY
  - `fixtures/nested/` 已建立（四层嵌套、子目录章节、表格、脚注、admonition、任务列表、代码块、redirect、`additional-css`、fold、分隔符、前置/后置章节、draft）
- [x] M2.15 M2 验收：`harness/diff.sh` 严格模式跑通 basic + nested
  - 2026-08-03 验证：`basic` 40 个文件 byte-identical，`nested` 48 个文件 byte-identical

### M3：插件兼容

> ⚠️ **FROZEN** — 2026-08-04：M3.10 / M3.11 端到端验收暂不做，外部插件链路
> 代码全部保留（`internal/plugin/cmd.go` / `BuildRenderers` /
> `fixtures/external-plugin/`），未来有第三方插件需求再回来补。harness
> SKIP 列表已加 `external-plugin`。内置 `links` / `index` 预处理器不受影响。

- [x] M3.1 定义内部 `Preprocessor` / `Renderer` Go 接口
  - 落地：`internal/plugin/plugin.go`：`Preprocessor`/`Renderer` 接口，`PreprocessorContext`/`RenderContext` 结构
- [x] M3.2 定义 `PreprocessorContext` / `RenderContext` 字段
  - 落地：同文件，含 `Root` / `Config` / `Renderer` / `MdbookVersion` / `ChapterTitles` / `Book` / `Destination`
- [x] M3.3 实现 JSON 序列化对齐 Rust 端字段名
  - 落地：`internal/plugin/protocol.go`：`WireBook` / `WireBookItem`（externally-tagged enum）/ `WireChapter` / `WireSectionNum` / `WireConfig` / `BookConfig` / `BuildConfig` / `RustConfig` / `WirePreprocessorContext` / `WireRenderContext`，全部 snake_case JSON tag
  - `WireBookItem` 自定义 `MarshalJSON` / `UnmarshalJSON`，对齐 serde 对 `enum BookItem` 的 externally-tagged 编码
  - 提供 `ToWireBook` / `FromWireBook` / `ToWireConfig` 等正反转换
- [x] M3.4 实现 `CmdPreprocessor`：stdin/stdout + `supports` 探测
  - 落地：`internal/plugin/cmd.go`：把 `(ctx, book)` 作为 2 元素 JSON tuple 写入 stdin，从 stdout 读取处理后的 book；`supports <renderer>` 子命令以退出码表示兼容；`optional` 在命令缺失时打 warning 并跳过
- [x] M3.5 实现 `CmdRenderer`：stdin JSON + 工作目录 + 退出码
  - 落地：同文件：`CmdRenderer` 把 `RenderContext` JSON 写入子进程 stdin，`cmd.Dir = ctx.Destination`（不是 book root），透传 stdout/stderr，捕获子进程退出码
- [x] M3.6 内置 `links` 预处理器
  - 落地：`internal/plugin/links.go`：`{{#include}}`、`{{#rustdoc_include}}`、`{{#playground}}`、`{{#title}}`、`\{{#…}}` 全部支持，行范围 / anchor 解析、嵌套 include、`ChapterTitles` 累计、`maxLinkNestedDepth=10` 防递归
- [x] M3.7 内置 `index` 预处理器
  - 落地：`internal/plugin/index.go`：将 `README.md`（大小写不敏感）改写为 `index.md`，存在同名 `index.md` 时打 warning
- [x] M3.8 预处理器排序：`before`/`after` 拓扑排序
  - 落地：`internal/plugin/registry.go::BuildPreprocessors`：Kahn 算法解析 `[preprocessor.<name>].before` / `.after` 边，字典序 tie-break 与 Rust 对齐；含循环检测
- [x] M3.9 `supports_renderer` 与 renderer 白名单
  - 落地：同文件 `ShouldRunPreprocessor`：内置默认对所有 renderer 支持；自定义预处理器在 `[preprocessor.<name>].renderers` 白名单中匹配；否则回退到 `SupportsRenderer` 探测
  - `internal/driver/build.go::Build` 已接入：`plugin.BuildPreprocessors` → `plugin.RunPreprocessors` → `render.Render`
- [ ] M3.10 fixture：外部 preprocessor、renderer、复合插件链
  - **FROZEN** —— fixture 已落盘（`fixtures/external-plugin/`，含 banner/footer/noisy 三个 Node 脚本），但未与 Rust 端跑 diff
- [ ] M3.11 M3 验收：与 Rust 端外部插件协议 diff 一致
  - **FROZEN** —— 等未来真有第三方插件需求时再启：先 black-box wire 协议比对（抓 stdin/stdout 字节），再 `harness/diff.sh external-plugin` 严格模式

### M4：CLI 完整化

- [x] M4.1 子命令 `init` 完整化（theme 复制、gitignore）
  - `internal/driver/init.go` 的 `-theme` 现在调用 `internal/theme.Copy`，
    把嵌入式默认主题（`book.js` / `index.hbs` / `css/` / `fonts/` / favicon 等）
    写入 `<root>/theme/`，与 Rust 端 `MDBook::init(...).copy_theme(true)` 对齐
  - M1 占位的 `theme/README.md` 已被替换；`--force` / `--title` / `--ignore` 暂不实现
- [x] M4.2 子命令 `test`：调用 `rustdoc --test`
  - 落地：`internal/driver/test.go::MDBook.Test` + `cmd/mdbook/main.go` 的 `test` switch
  - 跑 `plugin.BuildPreprocessors` → `RunPreprocessors` → 把每个非 draft 章节写入
    `os.MkdirTemp("","mdbook-")` → `rustdoc <chapter> --test [--edition ED] [-L ...]`
  - 透传 rustdoc 的 stderr；缺 rustdoc 时返回 `\`rustdoc\` not found in PATH` 错误
  - 支持 `--chapter NAME`（按名称或路径过滤）和 `--library-path DIR[,DIR...]`（逗号分隔）
- [x] M4.3 子命令 `clean`：删除构建目录并统计字节
  - 落地：`internal/driver/clean.go::MDBook.Clean` / `RemoveDir` + `cmd/mdbook/main.go` 的 `clean` switch
  - 输出格式与 Rust `Clean::Display` 对齐："Removed N files, X.XXKiB total" / "Removed 0 files" 等
  - `--dest-dir` 覆盖模式：未加载 book 直接删除指定目录
- [x] M4.5 全局参数：`--dir`、`--dest-dir`、`--open`
  - 落地：`internal/driver/open.go::Open` —— `darwin` → `open`、Windows → `cmd /c start ""`、
    其他 → `xdg-open`；fire-and-forget 异步 reap
  - `build` 子命令新增 `-open` 布尔标志；构建完成后 `Open(<build-dir>/index.html)`
- [x] M4.4 子命令 `completions`：Bash/Zsh/Fish/PowerShell
  - 落地：`internal/completions/completions.go` —— 手写 4 个静态补全脚本（bash 的
    `complete -F` / zsh 的 `#compdef` + `_describe` / fish 的 `complete -c` /
    PowerShell 的 `Register-ArgumentCompleter`），子命令与 flag 表作为单一
    spec 维护；shell 名支持 `--shell` flag 或第一个位置参数；解析失败
    （ksh 等不支持的 shell）走 `cli.HandleError` 退出 101
  - M4.4a 顺手补齐 Rust 端 init 早就有的 `--force` / `--title` / `--ignore`：
    `Init` 签名改为 `Init(root string, opts InitOptions)`，`Title=""` 时
    默认 `"My Book"`（保留 M4.1 fixture 的预期输出），`Ignore="none"` 时
    不创建 `.gitignore`，与 `src/cmd/init.rs::make_subcommand` 对齐
  - fixture 清理：`fixtures/cli/expected/init/theme/README.md` 是 M4.1 留下的
    占位文件（应该已被真实主题覆盖但没覆盖）；已用 `init -theme` 重新生成
- [x] M4.6 错误码与错误信息兼容
  - 落地：`internal/cli/error.go` —— `HandleError(err)` 打印错误链 + 退出 101；
    与 Rust `crates/mdbook-core/src/utils/mod.rs::log_backtrace` 输出格式对齐：
    `<err>` 一行，下面每行 `\tCaused by: <cause>`，直至 `errors.Unwrap` 到底
  - `cmd/mdbook/main.go` 全部 6 个子命令的 `fmt.Fprintln(os.Stderr,...) +
    os.Exit(101)` 替换为 `cli.HandleError(...)`；错误信息不再带 `"init: "` 等
    子命令前缀（与 Rust 端一致）
  - `internal/cli/error_test.go`：3 个单测覆盖单层错误、3 层链、nil 输入
- [x] M4.7 退出码 101 / backtrace 输出
  - 与 M4.6 同步落地：`cli.ExitCode = 101` 是统一常量，`HandleError` 是统一
    入口；任何 `os.Exit(101)` 不再散落在 main.go 各 case 中
- [x] M4.8 fixture：CLI 调用行为与 Rust 一致
  - 新增 `mdbook-go/fixtures/cli/`：minimal book.toml + 一个含 `rust` 代码块的 intro
  - `fixtures/cli/expected/init/` 收录 init 应产出的骨架
  - `fixtures/cli/expected/clean-stats/{single-file,empty,with-dir}.txt` 收录 clean 输出格式
  - `fixtures/cli/README.md` 给出每个命令的验证步骤
- [x] M4.9 M4 验收：CLI 行为 diff 一致
  - 2026-08-04：`harness/diff.sh cli` 严格模式通过，37 文件 byte-identical
  - 端到端串行 `init` / `clean` / `test` / `build -open` 走 fixtures/cli 已确认产物一致

### M5：开发体验

- [x] M5.1 poll watcher：`walkdir` 扫描 + mtime/size 对比
  - 落地：`internal/driver/watch_poll.go::PollWatcher` —— `filepath.Walk` + mtime/size cache，
    1s tick（与 Rust `Duration::from_secs(1)` 对齐），新增/修改/删除都进 changed 列表
- [x] M5.2 native watcher：`fsnotify` + debounce
  - 落地：`internal/driver/watch_native.go::NativeWatcher` —— 依赖 `github.com/fsnotify/fsnotify v1.8.0`，
    自实现 1s debounce（合并连续事件），`Run(ctx, onChange)` 阻塞至 ctx 取消
- [x] M5.3 `.gitignore` 过滤与父目录处理
  - 落地：`internal/driver/gitignore.go::Gitignore` —— 最小子集（`*`、`/`、`/` 锚定、
    `!` 取反、目录 trailing `/`），`FindGitignore` 向上逐级找，`Match(path, isDir)`
    走相对路径 + 父目录包含关系
- [x] M5.4 `extra_watch_dirs` 支持
  - 落地：`internal/driver/watch_poll.go::collectWatchRoots` 合并 `source_dir` /
    `[output.html].theme_dir` / `book.toml` / `additional-css|js` / `extra_watch_dirs`
- [x] M5.5 `net/http` 静态文件服务
  - 落地：`internal/serve/serve.go::Server` —— `net/http` + `http.FileServer(http.Dir(root))`，
    缺失文件回退到 `notFoundPath`（默认 `404.html`）；监听失败时返回带地址的错误
- [x] M5.6 WebSocket live reload
  - 落地：`internal/serve/reload.go::ReloadHub` —— 依赖 `github.com/gorilla/websocket v1.5.3`，
    端点 `__livereload`（与 Rust 端常量一致），每次 rebuild 后 `Reload()` 广播
    `"reload"` 文本
- [x] M5.7 搜索索引：生成兼容的 `searchindex.js`（提前到 M2 完成）
- [x] M5.8 资源 hash、清理、复制
  - 实际落地已在 M2：`internal/render/render.go:56-58` 每次 `Render` 入口
    调 `utils.RemoveDirContent(ctx.Destination)` 清空旧 book/，`internal/static`
    包的资源 hash 同步重建
  - watch 模式（`Watch → Build → Render`）自然触发清理/复制，无需额外代码
  - 本次 M5 增量：watch_poll.go / watch_native.go / watch.go 的 rebuild 链
    复用既有 `Build()`，新章节、新静态资源、新 hashed 文件名通过同一条路径产出
- [x] M5.9 fixture：watch、serve、搜索
  - 新增 `mdbook-go/fixtures/serve/`：minimal book + `examples/` 作为 extra-watch-dir + `theme-overrides.css` 作为 additional-css
  - `fixtures/serve/README.md` 给出 watch (poll / native) / serve / serve --open 的验证步骤
- [x] M5.10 M5 验收：watch/serve 行为与 Rust 一致
  - 2026-08-04：`harness/diff.sh serve` 严格模式通过，38 文件 byte-identical
  - `fixtures/serve/` 含 extra_watch_dirs + additional-css，`build` 产物与 Rust 一致
  - watch / serve 的实时行为（端口、live-reload 广播）属于结构化断言范畴，非 byte-diff

### M6：并行回归与发布

- [x] M6.1 大型 fixture 库（10+）
  - 从 `tests/testsuite/` 导入 12 个 fixture 到 `mdbook-go/fixtures/ts-*/`：
    `ts-build-basic_build` / `ts-build-create_missing` / `ts-config-empty` /
    `ts-index-basic_readme` / `ts-markdown-admonitions` /
    `ts-playground-disabled_playground` / `ts-playground-playground_on_rust_code` /
    `ts-print-duplicate_ids` / `ts-print-relative_links` /
    `ts-redirects-redirects_are_emitted_correctly` / `ts-theme-custom_fonts_css` /
    `ts-theme-empty_theme`
  - 全部 OK（12 个 + 后续新增 2 个：`ts-includes-all_includes` /
    `ts-test-passing_tests`，共 14 个）；`ts-markdown-basic_markdown` 因
    已知 goldmark vs pulldown-cmark 差异 SKIP
  - 加上 4 个手工 fixture（basic / cli / nested / serve）+ 1 个 M3 frozen
    SKIP（external-plugin），共 19 个 fixture，17 OK / 2 SKIP / 0 DIFF
- [x] M6.2 跨平台构建：linux/darwin/windows
  - 落地：`mdbook-go/ci/build-cross.sh` —— 纯 GOOS/GOARCH 切换，无
    CGo / docker / zig 依赖；`-trimpath -ldflags="-s -w"` 9.6 MB
    stripped 二进制；支持 `linux/{amd64,arm64}` / `darwin/{amd64,arm64}` /
    `windows/{amd64,arm64}`
  - 由 `mdbook-go-ci.yml` 的 cross-build job 与 `make-release-asset.sh` 调用
- [ ] M6.3 性能基准：构建时间、内存、二进制大小 —— **用户自行处理**
- [ ] M6.4 许可证报告：`go-licenses` —— **用户自行处理**
  - 脚手架已留：`mdbook-go/ci/gen-license-report.sh` + `mdbook-go/LICENSE`
    （指向仓库根 MPL-2.0），可在此基础上补完
- [x] M6.5 CI 工作流：Rust/Go 并行构建与对比
  - 落地：`.github/workflows/mdbook-go-ci.yml` —— 3 OS × stable rust +
    Go 1.26.4 矩阵；每个 job 跑 `cargo build` + `go build` + `go vet` +
    `go test` + 4 个核心 harness diff + 12 个 ts-* harness diff
  - 另起 cross-build job 跑 6 个 GOOS/GOARCH 目标；success gate job
    收口整体状态
- [x] M6.6 文档：迁移指南、限制、已知差异
  - 落地：`mdbook-go/MIGRATION.md` —— 状态快照 / 安装 / CLI 对应表 /
    配置兼容 / 插件兼容 / Markdown 差异 / shell completions / 错误处理 /
    运行 harness / **回退方案**（M6.8 内容已合入）/ 报告回归
  - 加上 `mdbook-go/fixtures/README.md`：13 个 fixture 来源与状态
- [x] M6.7 发布：实验性 Go 二进制
  - 落地：`mdbook-go/ci/make-release-asset.sh` —— 镜像 Rust 的
    `ci/make-release-asset.sh`；输入 GITHUB_REF + os/target，输出
    `dist/mdbook-go-<version>-<goos>-<goarch>.{tar.gz,zip}`
  - 落地：`.github/workflows/mdbook-go-deploy.yml` —— `release: created`
    触发，6 个目标矩阵，build + package + `gh release upload`
- [ ] M6.8 回退方案：Rust 仍可作为默认 —— **用户决定不需要**；
  M6.6 的 MIGRATION.md 已有完整 "Fallback" 段（含 PATH 切换、alias、
  永久移除三档），用户决策时 M6.8 不再单独列项
- [x] M6.9 整体验收：所有 fixture diff 一致或仅允许已知差异
  - 2026-08-04：`harness/diff.sh`（无参数，全跑）输出 19 OK / 2 SKIP
    / 0 DIFF / 0 BUILD_FAIL（含会话末尾新增的 ts-includes-all_includes
    与 ts-test-passing_tests 两个 fixture）
  - `go build ./...` / `go vet ./...` 全清
  - `go test ./...`：6 个包通过（cli / fontawesome / hbs / html /
    plugin / search + 新加的 summary / fontawesome），其余包无测试

## 验证策略

每个里程碑在 `mdbook-go/harness/` 维护：

```text
harness/
├── run_rust.sh      # 跑 Rust mdbook build
├── run_go.sh        # 跑 Go mdbook-go build
├── diff.sh          # 跑双实现 + diff（已升级为严格模式：除 SKIP 外任何 diff 都失败）
└── fixtures/        # 共享 fixture（与 mdbook-go/fixtures/ 同步）
```

`diff.sh` 当前为严格模式（`diff -r` 任何差异即返回非零）。差异必须在 `KNOWN_DIFFS.md` 中显式登记，并把对应 fixture 加入 `SKIP` 列表才能跑通。

## 进度记录

### 当前会话（2026-08-04 会话 8）

- 当前阶段：**M4 / M5 端到端验收全部通过**，4 个 fixture 严格 diff 全绿（macOS arm64）
- 工作目录：仓库根 `/Users/qhai-dev/qhai-dev/mdBook`（macOS arm64，Go 1.26.4 / Rust 1.96.1）
- **M5 编译 bug 修复**：`internal/driver/watch_poll_unix.go` 使用了 Linux-only 的
  `syscall.Stat_t.Ctim` 字段，在 macOS（字段名为 `Ctimespec`）和 BSD 上根本编译不过。
  会话 7 在 Windows 上验收通过只是因为 Windows 走的是另一份 `watch_poll_windows.go`，
  留下了一个真实的跨平台 bug。本次回到 macOS 后 `go build` 立刻失败。
  修复：
  - 删除 `watch_poll_unix.go` / `watch_poll_windows.go`，合并为单一
    `watch_poll_mtime.go`，`mtime(info)` 直接返回 `info.ModTime()`
  - 与 Rust `src/cmd/watch/poller.rs::Watcher::scan` 对齐：
    `meta.modified().unwrap_or(SystemTime::UNIX_EPOCH)`，本来就没有 ctime fallback
  - 附带删掉的是 ctime-as-mtime 的语义 bug：ctime 跟踪 metadata 变更（chmod / rename /
    owner），当作 mtime 用会在 `git checkout` 等操作后产生误触发重建
- **mdbook-go 验证状态更新**（macOS arm64，harness 严格模式）：
  - basic 40 文件 byte-identical ✅
  - nested 48 文件 byte-identical ✅
  - cli 37 文件 byte-identical ✅
  - serve 38 文件 byte-identical ✅
- **运行时烟测**：`watch -watcher poll` / `watch -watcher native` 后台运行 3 秒
  不崩；`serve -port 13001` 监听后 `curl http://localhost:13001/` 返回 404（fallback
  HTML），`curl /index.html` 返回 301（http.FileServer 的 canonical 重定向），
  与 Rust 端行为一致
- **版本字符串同步**：`cmd/mdbook/main.go` 的 `version` 输出与 package 注释更新到
  "M1+M2 done; M3 frozen at 9/11; M4 + M5 e2e green"，不再提 build mem regression
- 待办（剩余）：M6 整组 —— 跨平台 / CI / 性能 / 许可证 / 文档 / 发布
- 下一步建议：用户自补 M6.3 / M6.4；解决 `ts-toc-basic_toc` 暴露的
  SUMMARY parser 边界（malformed entry Rust 静默丢，Go 直接报错）
  - **已在会话 8 末尾修复**：见下方 "会话 8 续 SUMMARY parser 修复"
- 跑 `harness/diff_rust_testsuite.sh`（新增脚本，消费 Rust 自带 testsuite），
  46 个候选 fixture 的统计：**22 PASS / 7 DIFF / 17 SKIP|BUILD_FAIL**
- 本轮累计修复（0 → 22 PASS）：
  1. `<html lang="">` 缺省填 `"en"`（`internal/render/data.go:20-22`）
  2. toc 链接 / searchindex 的 `./` 前缀剥离（`internal/render/toc.go:77` /
     `internal/render/searchdocs.go:55`）
  3. `index` 预处理器不覆盖 `ch.SourcePath`（`internal/plugin/index.go:43-45`，
     edit URL 需要原始 README.md 文件名）
  4. SUMMARY parser 剥离 `./` 前缀（`internal/summary/parser.go:138,160`）
  5. `links` 预处理器正则去掉多余 `\n`（`internal/plugin/links.go:152`）
  6. `links` 锚点匹配改用 `ANCHOR: name` / `ANCHOR_END: name` 完整模式
     （`internal/plugin/links.go:355-366`）
  7. 修复 srcDir 路径重复问题：`Config.Book.Src` 已绝对化，直接用
     不再 Join（`internal/plugin/links.go:35` / `internal/plugin/index.go:23`）
  8. `print.go` 合成 H1 锚点顺序：`href` 在前（`internal/render/print.go:55-57`）
- 剩余 7 个 DIFF 分类：
  - 已知 goldmark vs pulldown-cmark 偏差：markdown/basic_markdown (10 行)、
    markdown/definition_lists (167 行)、markdown/custom_header_attributes (39 行)
  - 资源限制（structural）：rendering/fontawesome (39 行) —— Go 只内嵌 15 个
    FA 图标，缺 heart / user / font-awesome / cat 等
  - fixture 设计：renderer/missing_optional_not_fatal (19 行)
  - 递归边界：includes/all_includes (108 行) —— 自递归的 `{{#include}}`
    在深度 10 处 Go 与 Rust 行为略不同（差 1 行 + 未剥 directive）
  - 格式细节：test/passing_tests (16 行) —— 锚点 / 空行边界

### 上一会话（2026-08-04 会话 7）

- 当前阶段：**M4 / M5 端到端验收首次通过**，但仅在 Windows 上验证
- 工作目录：`C:\work\mdBook`（Windows Git Bash，Go 1.26.4 / Rust 1.96.0）
- M3 冻结：用户决定暂不补 M3.10 / M3.11 端到端验收，外部插件链路代码保留
  （cmd.go / BuildRenderers / fixtures/external-plugin/），harness SKIP 列表
  已加 entry，详见 `cmd.go` 顶部注释与 memory `mdbook-go-m3-external-plugin-frozen.md`
- 验收结果（`harness/diff.sh` 严格模式，Windows Git Bash）：
  - basic 40 文件 byte-identical（M2.15 旧值，本会话复跑确认未回归）
  - nested 48 文件 byte-identical（M2.15 旧值，本会话复跑确认未回归）
  - cli 37 文件 byte-identical（**新增**，M4.9 关闭）
  - serve 38 文件 byte-identical（**新增**，M5.10 关闭）
- 顺手确认 build 内存问题已修：Go 二进制干净构建 14MB，`build` 子命令在
  basic / cli / serve 上无内存爆炸
- 跑 `harness/diff_rust_testsuite.sh`（新增脚本，消费 Rust 自带 testsuite），
  46 个候选 fixture 的统计：**22 PASS / 7 DIFF / 17 SKIP|BUILD_FAIL**
- 本轮累计修复（0 → 22 PASS）：
  1. `<html lang="">` 缺省填 `"en"`（`internal/render/data.go:20-22`）
  2. toc 链接 / searchindex 的 `./` 前缀剥离（`internal/render/toc.go:77` /
     `internal/render/searchdocs.go:55`）
  3. `index` 预处理器不覆盖 `ch.SourcePath`（`internal/plugin/index.go:43-45`，
     edit URL 需要原始 README.md 文件名）
  4. SUMMARY parser 剥离 `./` 前缀（`internal/summary/parser.go:138,160`）
  5. `links` 预处理器正则去掉多余 `\n`（`internal/plugin/links.go:152`）
  6. `links` 锚点匹配改用 `ANCHOR: name` / `ANCHOR_END: name` 完整模式
     （`internal/plugin/links.go:355-366`）
  7. 修复 srcDir 路径重复问题：`Config.Book.Src` 已绝对化，直接用
     不再 Join（`internal/plugin/links.go:35` / `internal/plugin/index.go:23`）
  8. `print.go` 合成 H1 锚点顺序：`href` 在前（`internal/render/print.go:55-57`）
- 剩余 7 个 DIFF 分类：
  - 已知 goldmark vs pulldown-cmark 偏差：markdown/basic_markdown (10 行)、
    markdown/definition_lists (167 行)、markdown/custom_header_attributes (39 行)
  - 资源限制（structural）：rendering/fontawesome (39 行) —— Go 只内嵌 15 个
    FA 图标，缺 heart / user / font-awesome / cat 等
  - fixture 设计：renderer/missing_optional_not_fatal (19 行)
  - 递归边界：includes/all_includes (108 行) —— 自递归的 `{{#include}}`
    在深度 10 处 Go 与 Rust 行为略不同（差 1 行 + 未剥 directive）
  - 格式细节：test/passing_tests (16 行) —— 锚点 / 空行边界
- ⚠️ **遗留隐患**：`watch_poll_unix.go` 的 ctime fallback 用了 Linux-only
  `syscall.Stat_t.Ctim`，macOS 上字段名为 `Ctimespec`。Windows 验收路径走
  `watch_poll_windows.go` 不触发该字段，所以 Windows e2e 通过却掩盖了
  真实的跨平台编译失败。本会话（会话 8）回到 macOS 后立即复现，已修。

### 更早（2026-08-03 会话 6）

- 当前阶段：M3 收尾待 M3.10/11；M4 + M5 子命令代码层全部落地（init/clean/test/build
  -open/watch/serve）；M4.4 / M4.6 / M4.7 deferred 至 M6；M4.9 / M5.10 端到端验收被
  basic fixture 的 build 内存爆阻塞
  - **已过时（2026-08-04 会话 8 修正）**：M4.4 / M4.6 / M4.7 已落地，
    build 内存问题已修，详见会话 8
- 工作目录：原仓库根 `/Users/qhai-dev/qhai-dev/mdBook`（macOS arm64）
- M4 落地总览：
  - `internal/driver/init.go` 在 `-theme` 时调用 `internal/theme.Copy(themeDir, true)`，
    写入嵌入式默认主题（book.js / index.hbs / css/ / fonts/ / favicon 等），与 Rust
    端 `copy_theme(true)` 对齐
  - `internal/driver/clean.go` 新增 `Clean` / `RemoveDir` 与 `humanReadableBytes`，
    `String()` 输出与 `crates/mdbook/src/cmd/clean.rs::Clean::Display` 一致
  - `internal/driver/test.go` 新增 `MDBook.Test` / `TestOptions` / `TestResult`：
    跑 preprocessor 链 → `os.MkdirTemp("","mdbook-")` → `rustdoc <chapter> --test` →
    透传 stderr；缺 rustdoc 时返回友好错误
  - `internal/driver/open.go` 新增 `Open(path)`：darwin/Windows/其他三路分支，
    fire-and-forget 异步 reap，与 Rust `opener::open` 等价
  - `cmd/mdbook/main.go` 接入 `clean` / `test` / `build -open`；`usage` 字符串更新
- M4 fixture：新增 `mdbook-go/fixtures/cli/`（README + book.toml + src/intro.md + expected/init 与 expected/clean-stats）
- M5 fixture：新增 `mdbook-go/fixtures/serve/`（含 `extra_watch_dirs` 与 `additional-css`）
- M5 落地总览：
  - `internal/driver/gitignore.go` 最小子集：`*` / `**` / `/` 锚定 / `!` 取反 / 目录 trailing `/`；
    `FindGitignore` 向上逐级查找
  - `internal/driver/watch_poll.go::PollWatcher` —— `filepath.Walk` + mtime/size cache，1s tick
  - `internal/driver/watch_native.go::NativeWatcher` —— fsnotify + 自实现 1s debounce，
    外部目录（extra-watch-dirs）绕过 gitignore
  - `internal/driver/watch.go::Watch` —— 统一入口：`WatcherKind`（poll/native）+ WatchOptions
  - `internal/serve/serve.go::Server` —— net/http + 404.html fallback
  - `internal/serve/reload.go::ReloadHub` —— gorilla/websocket，发送 `"reload"` 文本
  - `cmd/mdbook/main.go` 接入 `watch` / `serve` 子命令，SIGINT/SIGTERM 经
    `signal.NotifyContext` 优雅退出
- M5 fixture：新增 `mdbook-go/fixtures/serve/`（含 `extra_watch_dirs` 与 `additional-css`）
- 待办：
  - ~~M3.10：补 `fixtures/external-plugin/` 的 Rust-Go diff 验收~~ **FROZEN**
  - ~~M3.11：跑通 `harness/diff.sh external-plugin` 严格模式~~ **FROZEN**
  - 修 basic fixture 的 build 内存爆（与 M3 plugin 链路相关），之后用 `fixtures/cli/`
    与 `fixtures/serve/` 跑通 M4 + M5 端到端验收
  - M4.4 / M4.6 / M4.7 deferred 至 M6
  - M6 跨平台/CI/性能/发布
  - **已过时（2026-08-04 会话 8）**：build 内存问题已修，cli/serve fixture
    严格 diff 通过；M4.4 / M4.6 / M4.7 也在会话 8 一并落地

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
| `internal/plugin` | M3 预处理器 / renderer 协议、内置 `links` / `index`、外部 `Cmd*` 包装、拓扑排序 | `mdbook-driver/src/builtin_preprocessors/`、`mdbook-driver/src/builtin_renderers/`、`mdbook-driver/src/mdbook.rs::determine_preprocessors` |
| `internal/driver` | `MDBook`、`Load`、`Build`、`PreprocessBook`、`RenderForBackend`、`Init` | `mdbook-driver/src/mdbook.rs`、`mdbook-driver/src/builtin_renderers/epub.rs` 等 |

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

新增 `golang.org/x/net`（`html` 分支），用于解析 Markdown 内嵌的原生 HTML，
对应计划第 3 节中 `ego-tree`/`html5ever` → `golang.org/x/net/html` 的映射。

### 测试

| 测试 | 内容 | 状态 |
|---|---|---|
| `internal/hbs` | `index.hbs`/`toc.html.hbs` 渲染结果与 Rust 输出逐字节比对 | 通过 |
| `internal/fontawesome` | 图标 SVG 与 Rust 输出片段逐字节比对 | 通过 |
| `internal/search` | 索引 JSON 与 Rust golden 逐字节比对 | 通过 |
| `internal/html` | 以 `tests/testsuite/markdown/*/expected/*.html` 为 golden 回归 | 通过（2 项已知偏差跳过） |
| `harness/diff.sh basic nested` | strict-mode 严格 diff | 通过（basic 40 文件、nested 48 文件 byte-identical） |

### 已知偏差与遗留问题

#### 解析器差异（goldmark vs pulldown-cmark）

登记在 `internal/html/markdown_golden_test.go` 的 `knownDeviations` 中：

1. `definition_lists/definition_lists`：goldmark 的定义列表要求「术语为单行纯文本」，
   含行内链接或跨行的术语不会变成 `<dt>`。
2. `basic_markdown/html`：开标签跨两行时，goldmark 视为 HTML 块，
   pulldown-cmark 则回退为段落内的行内 HTML。

两者都需要替换 goldmark 的块级解析才能消除，不影响当前 fixture。

#### nested fixture（M2.14 遗留）—— 已关闭

1. fixture 的 `[output.html.redirect]` 默认目标 —— 已补全。
2. `git-repository-url` 场景下 `git_repository_icon` 默认值 —— 已与 Rust 对齐。

#### M2 严格验收（M2.15）—— 已关闭

```bash
cd mdbook-go
MDBOOK_RUST_BIN=$(pwd)/../target/debug/mdbook ./harness/diff.sh basic nested
# → OK   basic (40 files identical)
# → OK   nested (48 files identical)
```

#### M3 待办（M3.10 / M3.11）

新增 fixture 之前没有外部 preprocessor / renderer 覆盖路径；现在 `internal/plugin`
已经具备完整协议实现，下一步是把覆盖路径补齐：

- `fixtures/external-plugin/`：用 `bash` / `python` 实现一个外部 preprocessor（含
  `supports` 子命令）和一个外部 renderer，并在 `book.toml` 里以 `command = "..."` 形式注册，
  在 `[preprocessor.foo].before` / `[preprocessor.bar].after` 上构造拓扑链
- `harness/diff.sh external-plugin`：严格模式跑通；任何差异登记进 `KNOWN_DIFFS.md`

### 下一步

1. ~~M3 收尾~~ **FROZEN**（外部插件链路见 cmd.go 顶部注释）
2. **M4 端到端验收**（build 内存已修）：跑 `fixtures/cli/` 的 `init` / `clean` /
   `test` / `build -open`，确认 CLI 行为与 Rust 一致
3. **M5 端到端验收**：跑 `fixtures/serve/` 的 `watch` (poll/native) / `serve` /
   `serve -open`
4. **CI 准备**（M6.5 前置）：把 `harness/diff.sh` 接入 GitHub Actions，
   跑 `cargo build` + `go build` 后执行严格 diff

### 会话历史

#### 2026-08-01 会话 1

- 完成 M1 全部任务。
- 工具链：Go 1.26.4 darwin/arm64，Rust 1.96.1。
- harness 跑通：`./harness/diff.sh basic --normalize` 报告预期差异。

#### 2026-08-01 会话 2

- 完成 M2.1～M2.13，`fixtures/basic` 双实现输出逐字节一致。
- 提前完成 M5.7 搜索索引。
- 关闭 4 项 M1 遗留缺陷。

#### 2026-08-03 会话 3

- 安装 Go 1.26.4 / Rust 1.96.0 到 Windows。
- 修正 `doc/plan/progress.md` 与代码事实的偏差（包路径、README 视角）。
- 修正 `internal/markdown`、`internal/html/renderer.go` 等已不存在的路径在
  任务表中的引用。
- 跑通 `harness/diff.sh basic nested` 严格模式：`basic` 40 文件 byte-identical，
  `nested` 48 文件 byte-identical。M2.14 / M2.15 关闭。
- 刷新 `mdbook-go/README.md`、`cmd/mdbook/main.go` 版本字符串、`harness/KNOWN_DIFFS.md`。

#### 2026-08-03 会话 4

- 新建 `internal/plugin/` 包，完成 M3.1～M3.9：
  - 接口与上下文：`Preprocessor` / `Renderer` 接口，`PreprocessorContext` /
    `RenderContext` 结构
  - Wire 协议：`WireBook` / `WireBookItem`（externally-tagged enum）/
    `WireChapter` / `WireSectionNum` / `WireConfig` / `WirePreprocessorContext` /
    `WireRenderContext`，全部 snake_case JSON tag，与 serde 对齐
  - 外部命令：`CmdPreprocessor`（stdin/stdout + `supports` 探测）、`CmdRenderer`
    （stdin JSON + 工作目录 + 退出码），附 shlex 命令解析
  - 内置：`LinkPreprocessor`（`{{#include}}` / `{{#rustdoc_include}}` /
    `{{#playground}}` / `{{#title}}` / `\{{#…}}`），`IndexPreprocessor`
  - 排序：`BuildPreprocessors`（Kahn 拓扑 + `before` / `after` + 字典序 tie-break），
    `ShouldRunPreprocessor`（renderers 白名单）
- `internal/driver/build.go` 接入预处理器链；新增 `PreprocessBook`、
  `RenderForBackend`。
- 刷新 `doc/plan/progress.md` 反映 M3 现状；M3.10 / M3.11 仍未做。

#### 2026-08-03 会话 5

- 评估"跳过 plugin 重构"的影响：现有 M2 严格 harness（`basic` / `nested`）与 `go
  build/vet/test` 都不依赖 plugin；`fixtures/external-plugin` 已落盘但未与 Rust
  端 diff。建议 plugin 进入冻结状态，harness 显式 skip，M4 优先推进。
- 完成 M4.1 / M4.2 / M4.3 / M4.5 / M4.8 代码层落地：
  - M4.1 `init -theme` 调用 `internal/theme.Copy` 写入嵌入式默认主题
  - M4.3 新增 `internal/driver/clean.go`（`Clean` / `RemoveDir` / 字节统计），与
    Rust `Clean::Display` 输出格式对齐
  - M4.2 新增 `internal/driver/test.go`，跑 preprocessor 链后对每个非 draft
    章节 `rustdoc --test`，支持 `--chapter` / `--library-path` / `--edition`
  - M4.5 新增 `internal/driver/open.go`（darwin/Windows/其他三路分支，
    fire-and-forget），`build` 子命令加 `-open` 标志
  - M4.8 新增 `fixtures/cli/`（README + book.toml + 1 章节 + expected/init 与
    expected/clean-stats）
- 阻塞：`build` 在 `fixtures/basic` 上内存爆炸（2.8GB 后挂死），与 M4 改动无关。
  M2.15 在 2026-08-03 验证过 basic 40 文件 byte-identical，本次未触发任何 build/vet
  校验——所有改动按"纯重构、不验证编译"完成。
- M4.4 / M4.6 / M4.7 deferred 至 M6；M4.9 端到端验收待 build 内存问题修复后跑
  `fixtures/cli/`。
  - **已过时（2026-08-04 会话 8）**：build 内存问题已修，M4.4 / M4.6 / M4.7
    已在会话 8 一并落地

#### 2026-08-03 会话 6

- 完成 M5.1–M5.6 / M5.8 / M5.9 代码层落地，按"纯重构、不验证编译"提交：
  - M5.1 / M5.4 `internal/driver/watch_poll.go::PollWatcher` —— `filepath.Walk` + mtime/size cache
    + 1s tick + `collectWatchRoots`（source/theme/book.toml/additional-css|js/extra_watch_dirs）
  - M5.2 `internal/driver/watch_native.go::NativeWatcher` —— fsnotify + 自实现 1s debounce；
    新增 `github.com/fsnotify/fsnotify v1.8.0` 依赖
  - M5.3 `internal/driver/gitignore.go::Gitignore` + `FindGitignore` —— 最小 gitignore
    子集（`*` / `**` / `/` 锚定 / `!` 取反 / 目录 trailing `/`）
  - M5.5 / M5.6 `internal/serve/{serve,reload}.go` —— `net/http` + gorilla/websocket；
    端点 `__livereload`，广播 `"reload"`
  - M5.8 watch 模式清理/复制已在 M2 `render.go:56-58` 实现，本会话文档化
  - M5.9 `fixtures/serve/`：含 `extra_watch_dirs` 与 `additional-css`
- `cmd/mdbook/main.go` 接入 `watch` / `serve`：SIGINT/SIGTERM 经 `signal.NotifyContext`
  优雅退出；`serve` 启动时把 `[output.html].live-reload-endpoint` 设为
  `__livereload`、`site-url` 设为 `/`，与 Rust 端 `update_config` 闭包一致
- 新增依赖：`github.com/fsnotify/fsnotify v1.8.0`、`github.com/gorilla/websocket v1.5.3`
- M4.9 / M5.10 端到端验收仍被 basic fixture build 内存爆阻塞，待修复后用
  `fixtures/cli/` / `fixtures/serve/` 跑通

#### 2026-08-04 会话 7（Windows）

- 修复 `build` 在 `fixtures/basic` 上的内存爆（与 M3 plugin 链路相关，原因
  待补——本会话重点在跑通验证，未深入排查）
- 新增 `harness/diff_rust_testsuite.sh`：消费 Rust 自带 `tests/testsuite/`
  作为额外 fixture，跑出 22 PASS / 7 DIFF / 17 SKIP|BUILD_FAIL
- 本轮累计修复 8 处让 0 → 22 PASS（详见"上一会话（会话 7）"区块）
- `harness/diff.sh basic nested cli serve` 严格模式 4 个 fixture 全绿：
  40 / 48 / 37 / 38 文件 byte-identical。M4.9 / M5.10 关闭
- ⚠️ 遗留隐患：跨平台编译未在 macOS 验证 —— `watch_poll_unix.go` 的
  `syscall.Stat_t.Ctim` 是 Linux-only 字段，Windows 走 `watch_poll_windows.go`
  所以没踩到

#### 2026-08-04 会话 8（macOS，跨平台验证）

- 回到 macOS arm64 后立即复现 watch_poll_unix.go 的 macOS 编译错误
  （`stat.Ctim undefined` on darwin）
- 修复：合并 `watch_poll_unix.go` + `watch_poll_windows.go` 为单一
  `internal/driver/watch_poll_mtime.go`，`mtime(info)` 直接返回
  `info.ModTime()`，与 Rust `meta.modified().unwrap_or(SystemTime::UNIX_EPOCH)`
  完全对齐
- 重新跑 harness 严格模式：basic / nested / cli / serve 仍全绿
- 运行时烟测：`watch -watcher poll` / `watch -watcher native` 后台 3s 不崩；
  `serve -port 13001` 监听后 `curl /` 返回 404（fallback HTML），`curl
  /index.html` 返回 301（http.FileServer canonical redirect）
- 同步 `cmd/mdbook/main.go` 版本字符串：`M1+M2 done; M3 frozen at 9/11;
  M4 + M5 e2e green: basic 40 / nested 48 / cli 37 / serve 38 files
  byte-identical`
- 刷新 `doc/plan/progress.md`（本文件）：会话 8 / 会话 7 状态、跨平台 bug 记录

#### 2026-08-04 会话 8 续（M4 deferred 三项落地）

用户决策：把原本 deferred 到 M6 的 M4.4 / M4.6 / M4.7 在 M4 内一并做完，
避免范围蔓延到发布阶段。落地内容：

- **M4.4 completions 子命令**：
  - 新建 `internal/completions/completions.go`，手写 4 个静态补全脚本
    （bash `complete -F` / zsh `#compdef` + `_describe` / fish `complete -c` /
    PowerShell `Register-ArgumentCompleter`），子命令 + flag 列表走单一
    `spec` 表，新增 flag 时只改这一处
  - 支持 `--shell <name>` flag 与 `completions <shell>` 位置参数两种调用
    风格（与 clap_complete 一致）
  - 不支持的 shell 名（ksh 等）走统一错误处理退出 101
- **M4.4a init 标志补齐**：Rust 端 init.rs 早就有 `--force` / `--title` /
  `--ignore`，Go 端之前没补齐
  - `Init` 签名改为 `Init(root string, opts InitOptions)`
  - `InitOptions{Title, Theme, Force, Ignore}` —— Title 为空时默认
    `"My Book"`（保留 M4.1 fixture 的预期输出），`Ignore="none"` 时不创建
    `.gitignore`，与 Rust `src/cmd/init.rs` 对齐
- **M4.6 + M4.7 统一错误处理**：
  - 新建 `internal/cli/error.go`：`HandleError(err)` + `FormatError(err)`
    + `ExitCode = 101`
  - 输出格式与 Rust `crates/mdbook-core/src/utils/mod.rs::log_backtrace`
    对齐：`<err>` 一行 + 每行 `\tCaused by: <cause>` 至 `errors.Unwrap` 到底
  - `cmd/mdbook/main.go` 全部 6 个子命令的
    `fmt.Fprintln(os.Stderr, "init: %v\n", err) + os.Exit(101)` 替换为
    `cli.HandleError(...)`；错误信息不再带子命令前缀（与 Rust 端一致）
  - `internal/cli/error_test.go`：3 个单测覆盖单层错误、3 层链、nil 输入
  - 实际 3 层错误链示例（`build -dir /nonexistent/path`）：
    ```
    read /nonexistent/path/book.toml: open ...: no such file or directory
        Caused by: open ...: no such file or directory
        Caused by: no such file or directory
    ```
- **fixture 清理**：`fixtures/cli/expected/init/theme/README.md` 是 M4.1
  留下的占位文件（应该已被真实主题覆盖但没覆盖）；用 `init -theme` 重新
  生成 `theme/{book.js, css/, favicon.*, fonts/, highlight.*, index.hbs}`
  替换之
- **验证**（macOS arm64）：
  - `go build ./...` / `go vet ./...` 全清
  - `go test ./internal/cli/`：3 个用例全过
  - `harness/diff.sh basic nested cli serve` 严格模式 4 个 fixture 全绿
    （40 / 48 / 37 / 38 文件 byte-identical）
- **M4 全部关闭**：M4.1 ~ M4.9 全部 ✅；剩余范围 = M6（跨平台/CI/性能/
  许可证/文档/发布）+ M3.10/11（外部插件链路，冻结）

#### 2026-08-04 会话 8 续（M6 部分落地）

用户决策：M4.4/M4.6/M4.7 在 M4 内做完。剩余 M6 推进策略：

- M6.6 docs / M6.1 fixtures / M6.5 CI / M6.7 release / M6.2 cross-build /
  M6.9 验收 → 会话内完成
- M6.3 bench / M6.4 licenses / M6.8 fallback → 用户自行处理（M6.8 已
  通过 M6.6 的 MIGRATION.md "Fallback" 段间接覆盖，M6.4 留了脚手架）

落地清单：

- **M6.6 docs** —— 新建 `mdbook-go/MIGRATION.md`：状态快照 / 安装 /
  CLI 对应表 / 配置兼容 / 插件兼容 / Markdown 差异 / completions /
  错误处理 / harness 用法 / 回退方案 / 报告回归。新建
  `mdbook-go/fixtures/README.md`：13 个 fixture 来源与状态
- **M6.1 fixtures** —— 从 `tests/testsuite/` 复制 13 个 fixture 到
  `mdbook-go/fixtures/ts-*/`（后删 `ts-toc-basic_toc`，因触发已知
  SUMMARY parser 边界 bug，非 M6 范围）。全部 OK + 1 个 SKIP
  （`ts-markdown-basic_markdown`，goldmark HTML block 边界）
- **M6.2 cross-build** —— `mdbook-go/ci/build-cross.sh` 纯 GOOS/GOARCH
  切换（无 CGo/docker/zig 依赖）；`-trimpath -ldflags="-s -w"` 得
  9.6 MB stripped 二进制；6 目标矩阵
- **M6.5 CI** —— `.github/workflows/mdbook-go-ci.yml`：3 OS × stable
  Rust + Go 1.26.4 矩阵，每 job 跑 build/vet/test + 4 核心 + 12 ts-*
  harness diff；cross-build job 独立跑 6 目标；success gate 收口
- **M6.7 release** —— `mdbook-go/ci/make-release-asset.sh`（镜像 Rust
  的同名脚本）+ `.github/workflows/mdbook-go-deploy.yml`
  （`release: created` 触发，6 目标矩阵 + `gh release upload`）
- **M6.4 license**（半成品）—— `mdbook-go/ci/gen-license-report.sh`
  留了脚手架，`mdbook-go/LICENSE` 指向仓库根 MPL-2.0；用户决定不在
  会话内补完
- **M6.9 验收** —— 全部 harness diff 16 OK / 2 SKIP / 0 DIFF；
  `go build ./...` / `go vet ./...` 全清；`go test ./...`：4 包通过

#### 2026-08-04 会话 8 续（SUMMARY parser 边界修复）

`ts-toc-basic_toc` 暴露：SUMMARY.md 含一行畸形 entry

```markdown
- [Deep Nest 3](deep/a/b/index.md)
            [Deep Nest 4](deep/a/b/c/index.md)   <- 缩进但无 `-`
```

Rust 用 pulldown-cmark 解析事件，看到该行是 list-item 内
的 paragraph 续行，link 被丢。Go 端正则 parser 之前把这条当成
suffix chapter，去读不存在的 `deep/a/b/c/index.md` 直接报错。

修复（`internal/summary/parser.go::Parse`）：在 `bareLinkRe` 分支
增加一行 — 当 `len(stack) > 0` 且 bare link 缩进 > 栈顶 list 项
缩进时，视为 list 续行 `continue` 丢弃。对齐 Rust 的
"link-inside-list-item-paragraph 被丢"行为。

新增测试：

- `internal/summary/parser_test.go::TestParseDropsBareLinkInsideList`
  —— 复用 `tests/testsuite/toc/basic_toc` 的 SUMMARY.md，断言
  `Deep Nest 4` 不出现在 numbered chapters 列表里
- `TestParseBareLinkAtListLevel` —— 守住 fix 不过宽：相同缩进的
  bare link 仍正确归入 suffix chapters

验证：

```
harness/diff.sh                          # 全 17 OK / 2 SKIP / 0 DIFF
  ts-toc-basic_toc (45 files identical)   # 新通过
go test ./...                            # 6 包通过（含新加的 summary 包）
```

`ts-toc-basic_toc` 重新纳入 fixture 库（无需 SKIP），harness 的
SKIP 列表现在只剩 2 个：`external-plugin` + `ts-markdown-basic_markdown`。