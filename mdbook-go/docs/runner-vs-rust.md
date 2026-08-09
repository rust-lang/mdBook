# Go `internal/runner` ↔ Rust `mdbook-driver` 对照

> 更新时间：2026-08-09，分支 v1。
> Rust 侧：`crates/mdbook-driver`（crate 结构见 `crate-mapping.md`）。
> 背景：`mdbook-driver` 是 Rust 的 "High-level library for running mdBook"；
> Go 侧原来叫 `internal/driver`，2026-08-09 更名为 `internal/runner`
> （命名缘由见 §4）。

**结论先行**：Go `internal/runner` 与 Rust `mdbook-driver` 在编排职责上
**1:1 对齐**——加载、构建、builtin preprocessor、preprocessor 注册表都在
同一包内，与 Rust 的 crate 边界一致（builtin 实现曾短暂放进 `internal/plugin`，
2026-08-09 迁回）。已知行为差异集中在 6 处（§3），其中 **create_missing 是
真实功能缺口**（Rust 有、Go 无），其余是默认值/命令裁剪/风格差异。

## 1. 结构对照

```
internal/runner/                crates/mdbook-driver/src/
├── loader.go          ↔       ├── load.rs
├── build.go           ↔       ├── mdbook.rs（MDBook::build）
├── registry.go        ↔       │   ├── determine_preprocessors
├── cmd.go             ↔       ├── builtin_preprocessors/cmd.rs
├── index.go           ↔       │   ├── index.rs
├── links.go           ↔       │   ├── links.rs
├── init.go            ↔       ├── init.rs
├── *_test.go          ↔       └── （Rust 单元测试）
```

命令层（`mdbook` bin `src/cmd/`）在 Go 侧放 `pkg/cmd/`：`init` → `create`、
`clean` → `clean`、`open` → `open`、`test` → **已删**（2026-08-09）。
watch 引擎（`src/cmd/watch/`）在 Go 侧为 `pkg/cmd/watch/`。

## 2. 逐文件对照表

| Go 文件 | Rust 对应 | 说明 |
|---|---|---|
| `loader.go` | `load.rs` | 加载流程 1:1，**但缺 `create_missing`（见 §3-1）** |
| `build.go` | `mdbook.rs::MDBook::build` | 编排主函数；`m.BuildDir()` 直接取配置值，无 `build_dir_for`（见 §3-2） |
| `registry.go` | `mdbook.rs::determine_preprocessors` / `preprocessor_should_run` | 1:1 |
| `cmd.go` | `builtin_preprocessors::cmd.rs` | 1:1（外部命令 preprocessor） |
| `index.go` | `builtin_preprocessors::index.rs` | 1:1 |
| `links.go` | `builtin_preprocessors::links.rs` | 1:1（含 `{{#include}}` / anchor 模式；`{{#rustdoc_include}}` 已随 Rust 遗留清理删除） |
| `init.go` | `init.rs` | 骨架一致，**默认值相反（见 §3-3）** |

Rust `builtin_renderers/`（markdown renderer）在 Go 侧**未移植**——Go 只有
HTML 渲染器（`internal/render`），无 `--format markdown` 路径。

## 3. 行为差异

### 3-1. `create_missing` 缺失（真实功能缺口）

Rust `load.rs:18-25`：`cfg.create_missing`（默认 `true`，config.rs:411）为
SUMMARY.md 中引用的缺失章节文件**自动补建**。Go 的 `[chapters]` 引用缺
失文件时同样未实现补建——只有配置字段（`internal/model/config.go:53`），
`loader.go` 无对应逻辑——Rust 会创建、Go 会报错。

> 优先级最高的偏离项（见 §5 建议 1）。

### 3-2. 多 renderer 与 `build_dir_for`

Rust `mdbook.rs:371` `build_dir_for`：多 renderer 时输出到
`build/<backend>/` 子目录（如 `build/html/`、`build/epub/`）。Go 只有单
HTML renderer，`build.go:35` 直接 `m.BuildDir()`，无 per-backend 子目录。

### 3-3. `init` 默认值相反

| | Rust `init.rs` | Go `init.go` |
|---|---|---|
| 源目录 | `src/` | `docs/` |
| 构建目录 | `book/` | `.doclens/` |
| `.gitignore` | 生成（`create_gitignore`，内容 `book/`） | **不生成**（2026-08-09 起删除，`--ignore` flag 同步移除） |
| 配置文件 | `book.toml` | `doclens.yaml`（含 `create-missing: true` 显式写出；目录树写 `[chapters]` 段，**不再生成 `SUMMARY.md`**） |

两边 init 产物对不上是**预期偏离**，fixtures 双文件（`book.toml` + `doclens.yaml`）
保持 harness 双腿一致——`SUMMARY.md` 同理保留，仅供 Rust 参考腿读取，Go 侧
只读 `doclens.yaml` 的 `[chapters]`；`tests/ts-config-empty/` 则显式声明
`root: src` / `build-dir: book` 以保住 Rust 默认语义的用例。

### 3-4. 已删部分（2026-08-09 及更早）

| 已删 | Rust 对应 | 原因 |
|---|---|---|
| `test` 命令 + `test.go` | `mdbook` bin `src/cmd/test.rs` | Go 未移植（用户要求移除） |
| `--ignore` flag | `init.rs` 的 `create_gitignore` | 见 §3-3 |
| playground / print / fontawesome | — | 更早的功能裁剪（见 git 历史） |

### 3-5. 风格差异

Rust 用 anyhow + env_logger（日志可见）；Go 用 `fmt.Errorf` 包装 + 静默
（错误走 `pkg/cmd` 的 `formatError` + exit 101）。

## 4. 命名：为什么叫 runner

Rust 用 "driver" 源于 **rustc_driver 惯例**（编译器生态中 "driver" = 顶层
编排库）。Go 侧 `driver` 没有这个典故支撑，语义不明（容易联想到设备驱动），
2026-08-09 更名为 **runner**（"运行器"）：它负责加载书、跑 preprocessor、
驱动渲染，是 mdBook 的编排运行器。契约层（接口 + wire 协议）留在
`internal/plugin`，对应 `mdbook-preprocessor` / `mdbook-renderer` 两个 crate。

## 5. 建议（按优先级）

| # | 事项 | 建议 |
|---|---|---|
| 1 | **create_missing 缺口**（§3-1） | 在 `loader.go` 实现 `create_missing`（对齐 load.rs:25-54），或至少在文档/错误信息中声明偏离 |
| 2 | 多 renderer / `build_dir_for`（§3-2） | 低优先：Go 单渲染器，无现实需求；如未来加 markdown renderer 再补 |
| 3 | `test` 命令（§3-4） | 低优先：Go 无测试执行需求；如对齐可加 `pkg/cmd/test.go` |
| 4 | `init` 交互 prompt（`init.rs` 有交互，Go `Force` 是空壳 flag） | 低优先：Go 全自动无 prompt 是刻意选择 |
| 5 | 日志可见性（§3-5） | 中优先：`env_logger` 式的 `--verbose` 输出对调试 harness 差异有用 |
