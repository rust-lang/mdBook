# doclens.yaml 配置参考

> 更新时间：2026-08-09，分支 v1。
> 代码位置：`internal/model/config.go`（加载与顶层默认）、`internal/model/html.go`
> （`DefaultHTML` / `DefaultSearch`）。
> Rust 基线：`crates/mdbook-core/src/config.rs`。

**结论先行**：`doclens.yaml` 是 `book.toml`（TOML）的 Go 端等价物，键名一致，
但格式为 YAML。默认值大部分与 Rust 基线一致，**刻意偏离**的有三处：
`package.root` 默认 `docs`（Rust 为 `src`）、`build.build-dir` 默认 `.doclens`
（Rust 为 `book`），以及 Go 端新增的若干字段（`build.pre-render`、
`output.html.mode`）。
`output.html.playground` / `output.html.print` 与 `[rust]` 相关配置已被硬删除，写了会被忽略。

## 1. 顶层结构

| 字段 | 类型 | 说明 |
|---|---|---|
| `package` | 表 | 书籍元信息 |
| `build` | 表 | 构建流程配置 |
| `chapters` | 表 | 目录树配置（2026-08-09 起取代 `SUMMARY.md`），见 §4 |
| `output` | 表 | 渲染器配置，键为渲染器名；目前仅 `html` 被实现，其余透传给插件 |
| `preprocessor` | 表 | 预处理器配置，键为插件名；Go 端不解析，由插件用自有 schema 解码 |

加载时缺失文件会报错；`preprocessor` / `output` 之外的表缺省取各段默认值。

## 2. `package` 段

| 字段 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `title` | string | 空 | 书名 |
| `description` | string | 空 | 描述 |
| `language` | string | `en` | ⚠️ Rust 基线默认 `Some("en")`；Go 端 `New()` 里未设（空串），有偏差 |
| `text-direction` | string | 空 | `ltr` / `rtl`；未写时按 `language` 推导（RTL 语言列表），否则 `ltr` |
| `root` | string | `docs` | ⚠️ **与 Rust 基线不同**（Rust 默认 `src`）；`SetRoot` 再兜底一次 |

## 3. `build` 段

| 字段 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `build-dir` | string | `.doclens` | ⚠️ **与 Rust 基线不同**（Rust 默认 `book`） |
| `extra-watch-dirs` | string[] | `[]` | watch/serve 时额外触发重建的目录 |
| `create-missing` | bool | `true` | `[chapters]` 引用的缺失章节文件是否自动创建（⚠️ Go 端当前未实现，见 `docs/runner-vs-rust.md` §3-1） |
| `pre-render` | string[] | `[]` | Go 端特有字段 |
| `use-default-preprocessors` | bool | `true` | 是否始终启用与渲染器兼容的默认 preprocessor |

## 4. `chapters` 段

目录树配置，2026-08-09 起取代 `SUMMARY.md`（Go 端不再读 `SUMMARY.md`）。
三个列表对应 mdBook summary 语法：`prefix` 排在编号章节之前、`numbered`
承载章节编号、`suffix` 排在最后。每一项（`ChapterItem`）恰好取一种形态：

| 形态 | 写法 | 说明 |
|---|---|---|
| 章节 | `name` + `path` | 普通章节；`path` 相对 `package.root`，前导 `./` 会被剥掉 |
| 草稿 | `name` + `path: ""` | 空路径的章节，**仍消耗一个编号**（对齐 Rust） |
| 部件标题 | `part` | 部件标题（对应 `# Part`），不消耗编号 |
| 分隔线 | `separator: true` | 对应 `---`，不消耗编号 |
| 嵌套 | `children` | 章节下的子章节列表，递归 |

```yaml
chapters:
  prefix:
    - name: Preface
      path: preface.md
  numbered:
    - part: Guide            # 部件标题，不消耗编号
    - name: Chapter 1
      path: chapter_1.md
      children:
        - name: Section 1.1  # 编号 1.1
          path: s1.md
    - name: Draft            # 草稿：消耗编号，不渲染正文
      path: ""
    - separator: true
  suffix:
    - name: Afterword
      path: afterword.md
```

**编号规则**（与 Rust 的 `toc.html` 输出逐字节对齐验证过）：只有
`numbered` 列表编号；顶层 1, 2, 3…，子级 1.1, 1.2…，逐层追加；
部件标题与分隔线不重置计数器；草稿仍占一个编号；`prefix` / `suffix`
不编号。

> 注意：fixture（`tests/`）中的 `SUMMARY.md` 文件仍保留，但仅供
> `harness/diff.sh` 的 Rust 参考腿使用（Rust mdbook 仍读
> book.toml + SUMMARY.md）。修改 fixture 目录树时须同时改两处。

## 5. `output.html` 段

默认值由 `DefaultHTML()` 提供（对齐 `impl Default for HtmlConfig`）。

| 字段 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `theme` | string | 空 | 主题目录；空 → `<root>/theme` |
| `default-theme` | string | 空 | 实际取 `light` |
| `preferred-dark-theme` | string | 空 | 实际取 `navy` |
| `mode` | string | `vim` | Go 端特有：键盘导航脚本模式，`vim`（h/l 导航章节）/ `normal`（仅方向键） |
| `smart-punctuation` | bool | `true` | 智能引号、省略号、en/em 破折号 |
| `definition-lists` | bool | `true` | 定义列表支持 |
| `admonitions` | bool | `true` | admonition 支持 |
| `mathjax-support` | bool | `false` | MathJax 支持 |
| `additional-css` | string[] | `[]` | 额外 CSS，注入 `<head>` |
| `additional-js` | string[] | `[]` | 额外 JS，注入 `<body>` 底部 |
| `fold.enable` | bool | `false` | 章节折叠 |
| `fold.level` | uint | `0` | 折叠层级 |
| `code.hidelines` | map<string,string> | `{}` | 语言 → 隐藏行前缀 |
| `no-section-label` | bool | `false` | 章节标签 |
| `search` | 表 | 见 §6 | 未写整表时用 §6 全部默认值 |
| `git-repository-url` | string | 空 | 仓库链接 |
| `git-repository-icon` | string | 空 | 实际取 `fab-github` |
| `edit-url-template` | string | 空 | "编辑本页"链接模板 |
| `input-404` | string | 空 | 404 页源文件，默认 `404.md` → 输出 `404.html`；**显式写空串禁用 404 页**（Go 端用指针区分"未写"与"写了空串"，见 `html.go` `Render404`） |
| `site-url` | string | 空 | 站点 URL（sitemap/robots 用） |
| `cname` | string | 空 | CNAME |
| `redirect` | map<string,string> | `{}` | 重定向映射 |
| `hash-files` | bool | `true` | 静态资源是否加内容 hash |
| `sidebar-header-nav` | bool | `true` | 侧边栏头部导航 |

**已硬删除的字段**（2026-08-07/09 提交移除，写了会被忽略）：
`playground`（含 `rust.edition` 的运行时用途）、`print`、`[rust]` 表、
`package.authors`、`package.multilingual`。

## 6. `output.html.search` 段

仅当用户写了 `search:` 表时才解析；表内未写的键仍取默认（`DefaultSearch()`，
与 `mdbook-search` 的默认一致）。整表未写时走 `EffectiveSearch()`，同样取默认。

| 字段 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `enable` | bool | `true` | 是否启用搜索 |
| `limit-results` | int | `30` | 结果数上限 |
| `teaser-word-count` | int | `30` | 摘要字数 |
| `use-boolean-and` | bool | `false` | 布尔 AND |
| `boost-title` | int | `2` | 标题权重 |
| `boost-hierarchy` | int | `1` | 层级权重 |
| `boost-paragraph` | int | `1` | 段落权重 |
| `expand` | bool | `true` | 结果是否展开 |
| `heading-split-level` | int | `3` | 按标题拆分的层级 |
| `copy-js` | bool | `true` | 是否拷贝搜索 JS |
| `chapter.<name>.enable` | bool | `false` | 按章节覆盖搜索启用状态 |

## 7. 调整配置时的坑

1. **默认值语义**：`output.html` 的解析在 `config.go` 的 `HTML()` 中通过
   YAML round-trip 实现（相当于 serde 的 `#[serde(default)]`），未写的键保留
   默认值、不会被零值覆盖。因此**想关掉默认开启的项（如 `admonitions`、
   `hash-files`）必须显式写 `false`**。
2. **`root` / `build-dir` 与 Rust 默认不同**：想沿用 mdbook 习惯的
   `src` / `book` 需显式配置，例如 fixture（`tests/basic/doclens.yaml`）：
   ```yaml
   package:
     root: src
   build:
     build-dir: book
   ```
3. **插件配置透传**：`output.*` / `preprocessor.*` 除 `html` 外均为原始
   yaml 值，插件自行解码；键名冲突由插件 schema 决定，Go 端不校验。
4. **404 页的特殊语义**：`input-404` 不写 = 默认生成 404 页；写空串 = 禁用。
5. **环境变量**：Rust 端支持 `MDBOOK_*` 覆盖（`config.rs` `update_from_env`），
   Go 端当前未实现，调整配置只能改文件。

## 8. 最小示例

```yaml
package:
  title: My Book
  language: zh
  root: src

build:
  build-dir: book
  create-missing: true
  use-default-preprocessors: true

chapters:
  prefix:
    - name: Introduction
      path: intro.md
  numbered:
    - name: Chapter 1
      path: chapter_1.md

output:
  html:
    default-theme: light
    preferred-dark-theme: navy
    mode: vim
    smart-punctuation: true
    definition-lists: true
    admonitions: true
    mathjax-support: false
    additional-css:
      - custom.css
    hash-files: true
    search:
      enable: true
      limit-results: 30
      boost-title: 2
```
