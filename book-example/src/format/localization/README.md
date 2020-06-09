# Localization

MDBook uses [fluent] as the backend for its localisation system. You can read
more about how to use Fluent's syntax in the [Fluent Syntax Guide][flg].  You
can enable localisation by setting `multilingual = true` in your `book.toml`.

[fluent]: https://projectfluent.org
[flg]: https://projectfluent.org/fluent/guide/

## Directory structure

By default MDBook will look for localizations and Fluent syntax files in
the `locales` directory. You can override this by setting `locales = "/path"` in
your `book.toml`. MdBook will automatically detect any directory that is named
after a valid [Unicode Language Identifier][uls] such as `fr`, `en-US`,
or `en-latn-US-Valencia`.

### Config
```toml
multilingual = true
locales = ["locales"]
```

### Example Locales Directory
```
locales
├── core.ftl
├── en
│   └── main.ftl
└── fr
    └── main.ftl
```

[uls]: https://tools.ietf.org/html/rfc4646

## Shared fluent files
Sometimes you need to share fluent files across different locales. You can
specify additional files that shared with the `shared-locale-resources`
property in your `book.toml`

### Example
```toml
shared-locale-resources = ["locales/core.ftl"]
```

## Localizing your MDBook
You can call fluent by using {{{{raw}}}}`{{fluent}}` or `{{#fluent}}`{{{{/raw}}}} syntax in your
markdown (or HTML if you're building a theme).

### Markdown File
{{{{raw}}}}
```markdown
{{#include ./example.md}}
```
{{{{/raw}}}}

### Fluent (en)
{{{{raw}}}}
```markdown
{{#include ../../../locales/en/main.ftl}}
```
{{{{/raw}}}}

### Fluent (fr)
{{{{raw}}}}
```markdown
{{#include ../../../locales/fr/main.ftl}}
```
{{{{/raw}}}}

### {{fluent "localized-chapter-title"}}

- `simple`: {{fluent "simple"}}
- `reference` {{fluent "reference"}}
- `parameter(param=Ferris)` {{fluent "parameter" param="Ferris"}}
- `crab`
    - `$crabs=1` {{fluent "crabs" crabs=1}}
    - `$crabs=10` {{fluent "crabs" crabs=10}}
- `fallback` {{fluent "fallback"}}

## Shared fluent files
Sometimes you need fluent files that shared 
