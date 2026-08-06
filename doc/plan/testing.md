# mdbook-go 测试基线与 Rust 兼容性追踪

> 本文件是「Rust 端 `tests/testsuite/` 全套回归」与「mdbook-go 是否可以删除 Rust 源码」这两个问题的唯一权威记录。
> 跑测入口见 [§1 运行方式](#1-运行方式)；删除 rust 前的硬性条件见 [§6 删除前验收清单](#6-删除前验收清单)。

---

## 1. 运行方式

```bash
# 1. 编译两端的二进制（首次或改了源码时）
cd /c/work/mdBook
cargo build --bin mdbook                         # Rust 端 → target/debug/mdbook.exe
(cd mdbook-go && go build -o bin/mdbook-go.exe ./cmd/mdbook)

# 2. 跑 Rust testsuite 全套（自动发现 tests/testsuite/<cat>/<name>/book.toml）
bash mdbook-go/harness/diff_rust_testsuite.sh

# 3. 只跑 mdbook-go 本地 fixture（22 个）
bash mdbook-go/harness/diff.sh
```

Harness 会自动：

- 跳过没有 `src/SUMMARY.md` 的 fixture
- 跳过 `[preprocessor.*].command` 用外部命令的 fixture（M3 冻结）
- 跳过 `expect_failure()` 标注的 Rust 测试
- 输出 PASS / DIFF / SKIP / BUILD_FAIL 四档汇总

---

## 2. 测试语料

| 来源 | 路径 | 数量 | 用途 |
|---|---|---|---|
| Rust 端 testsuite | `tests/testsuite/{build,cli,config,includes,index,init,markdown,playground,preprocessor,print,redirects,renderer,rendering,search,test,theme,toc}/*/` | 47 candidates | 端到端对照基线（权威） |
| mdbook-go 本地 fixture | `mdbook-go/fixtures/*/` | 22 | Go 端独立 fixture，含从 Rust 导入的 ts-* |

---

## 3. 最近一次完整回归（2026-08-05，删 rust 前的基线扫描）

```
SUMMARY: 25 PASS, 5 DIFF, 6 SKIP, 12 BUILD_FAIL  (of 47 candidates)
```

### 3.1 PASS（25）— 字节级与 Rust 输出一致

```
build/basic_build                                  37 files
build/create_missing                               37 files
config/empty                                       37 files
includes/all_includes                              46 files
index/basic_readme                                 39 files
markdown/admonitions                               37 files
playground/disabled_playground                     37 files
playground/playground_on_rust_code                 37 files
print/chapter_no_h1                                39 files
print/duplicate_ids                                38 files
print/relative_links                               39 files
redirects/redirects_are_emitted_correctly          40 files
rendering/code_blocks_fenced_with_indent           37 files
rendering/default_rust_edition                     37 files
rendering/edit_url_template                        36 files
rendering/editable_rust_block                      42 files
rendering/header_links                             37 files
rendering/hidelines                                37 files
rendering/html_blocks                              38 files
search/disable_search_chapter                      40 files
test/failing_tests                                 39 files
test/passing_tests                                 42 files
theme/custom_fonts_css                             24 files
theme/empty_fonts_css                              22 files
toc/basic_toc                                      45 files
```

### 3.2 DIFF（5）

详见 [§4 gap 清单](#4-gap-清单)。

```
markdown/basic_markdown                  10 diff lines
markdown/custom_header_attributes        39 diff lines
markdown/definition_lists               167 diff lines
renderer/missing_optional_not_fatal     19 diff lines
rendering/fontawesome                   39 diff lines
```

### 3.3 SKIP（6）— 全部合理

```
preprocessor/extension_compatibility        M3 frozen (external preprocessor)
preprocessor/failing_preprocessor           M3 frozen
preprocessor/missing_optional_not_fatal     M3 frozen
preprocessor/missing_preprocessor           M3 frozen
preprocessor/nop_preprocessor               M3 frozen
rendering/edit_url_template_explicit_src    no src/SUMMARY.md
```

### 3.4 BUILD_FAIL（12）— Rust 端构建失败，Go 端多数表现良好

按根因分组：

**(a) Go 端静默成功（应修，见 [§4 A 类](#a-类-silent-validation-gap---必修)）**

```
build/no_reserved_filename
redirects/redirect_existing_page
search/chapter_settings_validation_error
theme/empty_theme
theme/missing_theme
```

**(b) Rust 端正向错误用例，Go 也正确失败（已对齐）**

```
build/missing_file                        Go exit=101 ✓
redirects/redirect_removed_with_fragments_only  Go exit=101 ✓
rendering/fontawesome_error               Go exit=101 ✓
```

**(c) Rust 端缺外部 binary，本机复现限制**

```
renderer/backends_receive_render_context_via_stdin    缺 ./cat-to-file
renderer/missing_renderer                            缺自定义 backend
renderer/renderer_with_arguments                     缺 arguments backend
```

---

## 4. gap 清单

### A 类 — Silent validation gap — 必修

> Rust 拒绝、Go 静默成功。删除 Rust 源码后会变成「上线才发现错」。

| # | Fixture | book.toml 关键内容 | Rust 期望错误 | Go 现状 |
|---|---|---|---|---|
| A1 | `build/no_reserved_filename` | src 含 `print.md` | `print.md is reserved for internal use` | exit 0，输出正常 HTML |
| A2 | `redirects/redirect_existing_page` | `[output.html.redirect] "/chapter_1.html" = "other-page.html"` 且 src 已有 `chapter_1.md` | `redirect found for existing chapter at /chapter_1.html` | exit 0，输出正常 HTML |
| A3 | `search/chapter_settings_validation_error` | `[output.html.search.chapter] "does-not-exist" = { enable = false }` | `key 'does-not-exist' does not match any chapter paths` | exit 0，输出正常 HTML |
| A4 | `theme/missing_theme` | `[output.html] theme = "./non-existent-directory"` | `theme dir ./non-existent-directory does not exist` | exit 0，fallback 默认主题 |
| A5 | `theme/empty_theme` | `[output.html] theme = "./theme"`，fixture 无 theme dir | 同 A4 | exit 0，fallback 默认主题 |

**修复入口**（待补实现）：

- A1 — `internal/book/model.go` 增加 reserved 文件名黑名单（`print.md` / `404.md`）
- A2 — `internal/config/config.go` 或构建阶段，验证 redirect target 不冲突
- A3 — `internal/config/config.go` 解析 `search.chapter` 后对照 `Book` 树
- A4/A5 — `internal/config/config.go` 解析 `output.html.theme` 后 `os.Stat` 校验

### B 类 — Functional gap — 强烈建议修完再删

> Go 输出与 Rust 不同但都成功；用户视角会看到不一致。

| # | Fixture | 差异本质 |
|---|---|---|
| B1 | `markdown/custom_header_attributes` | Go 完全忽略头属性语法 `{#id .class key=value}`。Rust 产出 `<h2 id="myh3" class="myclass1 myclass2">`，Go 产出 `<h2 id="heading-with-attribute...">`（所有属性串接到 id） |
| B2 | `markdown/definition_lists` | Go 把 `term: def` 渲染成 `<p><a>term</a>: def</p>`；Rust 正确产出 `<dl><dt>term</dt><dd>def</dd></dl>`；嵌套定义列表彻底坏 |
| B3 | `renderer/missing_optional_not_fatal` | Go 没实现 `output.X.optional = true` 标志。Rust 在 backend 缺失且 optional 时 warn 并跳过；Go 总是 fail 或正常渲染 |

**修复入口**（待补实现）：

- B1 — `internal/markdown/parser.go` 加头属性后处理（goldmark 不原生支持，需在 AST 上识别 `{#...}` 后缀并改写属性）
- B2 — 验证 `goldmark` 是否启用 `extension.DefinitionList`；若已启用，问题在 title-id 生成对 dt/dd 的处理
- B3 — `internal/config/config.go` 解析 `output.X.optional` 并在 renderer 调度时短路

### C 类 — 已知差异 — 可保留，归档 KNOWN_DIFFS

| # | Fixture | 差异 |
|---|---|---|
| C1 | `markdown/basic_markdown` | `<meta>` 是否被 `<p>` 包裹：goldmark 不包，pulldown-cmark 包。已在 `MIGRATION.md` "Markdown / HTML differences" 一节登记 |
| C2 | `rendering/fontawesome` | Go 输出 `<i class="fas fa-heart">`；Rust 输出内联 SVG。`internal/fontawesome/fontawesome.go` 在 v0.2 已标 `# Deprecated`，stderr 一次提示迁移 |

### D 类 — Serve / runtime gap — 修于 2026-08-05 后半段（v0.2 后第二轮 smoke test 发现）

> 不在 `diff_rust_testsuite.sh` 范围内——它是运行 `mdbook-go serve` 后浏览器/HTTP 客户端拿到的端点行为差异。和 Rust 端对照基线是 `tower_http::services::ServeDir` 行为。

#### D1 ✅ 已修（2026-08-05）

**症状**：`mdbook-go serve` 对 `/index.html`、`/nested/index.html`、`/deep/a/b/index.html` 等以 `index.html` 结尾的 URL 返 **301 → ./ → 404**，所有嵌套 index 章节页都打不开。

**根因**：Go 标准库 `net/http/fs.go:685`：

```go
func serveFile(w ResponseWriter, r *Request, fs FileSystem, name string, redirect bool) {
    const indexPage = "/index.html"
    if strings.HasSuffix(r.URL.Path, indexPage) {
        localRedirect(w, r, "./")
        return
    }
```

`http.FileServer` 内部调用 `serveFile`，**任何以 `/index.html` 结尾的 URL 都被强制 301 到 `./`**。这是 stdlib 的设计——把 `index.html` 当"目录索引"，要求用户访问目录 URL 而不是显式文件名。Rust `ServeDir` 没这个怪癖，mdBook 的章节 URL 字面就是 `index.html`，撞上了。

**修复**：重写 `internal/serve/serve.go::staticHandler`，**不再用 `http.FileServer`**，改用 `os.Stat` + `os.Open` + `io.Copy`，匹配 Rust `ServeDir` 行为。

**改动文件**：
- `mdbook-go/internal/serve/serve.go` — 重写 staticHandler（约 60 行），新增 `resolveStaticPath` / `serveNotFound` 辅助函数
- `mdbook-go/internal/serve/serve_test.go` — **新增**，第一个 serve 包测试（7 个 URL 类别 + 路径解析单元测试）

**验证**：
- `go test ./internal/serve/ -v` → 全部 PASS
- `go test ./internal/...` → 全部 PASS（无回归）
- smoke test：`tests/testsuite/toc/basic_toc` fixture + `serve -port 3001`
  - `/index.html` / `/nested/index.html` / `/deep/a/b/index.html` / `/` → **200**（之前 301 → 404）
  - `/prefix1.html` / `/toc.html` / `/print.html` / `/css/*.css` / `/toc-{hash}.js` / `/favicon*` → **200**（未破坏）
  - `/nonexistent.html` → **404** with `404.html` body（保留）
- 截图：`tmp/serve-index-fixed.jpeg`、`tmp/serve-deep-ab-fixed.jpeg`

---

## 5. 复现命令（随手验证用）

```bash
# A 类 5 处：观察 Go 是否静默成功
REPO=/c/work/mdBook
GO=$REPO/mdbook-go/bin/mdbook-go.exe
for fx in build/no_reserved_filename \
          redirects/redirect_existing_page \
          search/chapter_settings_validation_error \
          theme/empty_theme \
          theme/missing_theme; do
  echo "=== $fx ==="
  "$GO" build -dir "$REPO/tests/testsuite/$fx" -dest-dir /tmp/xx >/dev/null 2>&1
  echo "exit=$?"
done

# B 类 3 处：产出 diff
mkdir -p /tmp/regress && rm -rf /tmp/regress/*
for fx in markdown/custom_header_attributes \
          markdown/definition_lists \
          renderer/missing_optional_not_fatal; do
  "$REPO/target/debug/mdbook.exe" build "$REPO/tests/testsuite/$fx" --dest-dir /tmp/regress/$fx/rust >/dev/null 2>&1
  "$GO" build -dir "$REPO/tests/testsuite/$fx" -dest-dir /tmp/regress/$fx/go >/dev/null 2>&1
  diff -r /tmp/regress/$fx/rust /tmp/regress/$fx/go | head -30
done
```

---

## 6. 删除前验收清单

### 🚫 当前状态：**不通过** — A/B 类 8 处仍未修；D1 已修

| 类别 | 数量 | 必修？ | 当前状态 |
|---|---|---|---|
| A 类 silent validation | 5 | ✅ 必须全部修复 | 🔴 未修 |
| B 类 functional | 3 | ✅ 强烈建议全部修复 | 🔴 未修 |
| C 类 known diff | 2 | ❌ 可保留 | 🟡 登记完成 |
| D 类 serve / runtime | 1 | ✅ v0.2 后已修 | 🟢 D1 ✅ |

### 删除 Rust 源码的硬性条件

1. **A 类 5 处全部修复**，且 `bash mdbook-go/harness/diff_rust_testsuite.sh` 中 `build/no_reserved_filename`、`redirects/redirect_existing_page`、`search/chapter_settings_validation_error`、`theme/empty_theme`、`theme/missing_theme` 这 5 个从 BUILD_FAIL 转为 PASS（或至少从 BUILD_FAIL 转为 Go 也对应 exit 101）
2. **B 类至少修完 B1、B2**（头属性 + 定义列表），否则 Go 的 markdown 输出对 Rust 文档会肉眼可见的不一致
3. `MIGRATION.md` 新增一节 "Breaking differences vs Rust"，列清楚不支持的特性
4. 把 `tests/testsuite/` 保留为 read-only 拷贝（不放 cargo 也不放构建脚本），作为日后的回归对照基线
5. `progress.md` 加一条："Rust 源码删除，回归基线已迁移至 doc/plan/testing.md"

### 验收命令

```bash
bash mdbook-go/harness/diff_rust_testsuite.sh | tee /tmp/regression-final.txt
# 期望：
# - 25 PASS 维持
# - 5 个 A 类 fixture 不再 BUILD_FAIL（要么 PASS，要么单独 SKIP 并注释）
# - 5 个 DIFF 中至少 B1+B2 消失
```

---

## 7. 变更日志

- 2026-08-05（前半段）：v0.2 后第一轮全量扫描，47 candidates → 25 PASS / 5 DIFF / 6 SKIP / 12 BUILD_FAIL。识别 A 类 5 处 / B 类 3 处 / C 类 2 处。
- 2026-08-05（后半段）：起 `mdbook-go serve` smoke test，发现 D1（Go stdlib `http.FileServer` 硬编码 `/index.html → ./` 301 redirect，撞 mdBook 字面 `index.html` 章节 URL）。重写 `internal/serve/serve.go::staticHandler` 不再使用 `http.FileServer`，加 `internal/serve/serve_test.go`（首个 serve 包测试），全套测试通过。识别 D 类作为新分类。