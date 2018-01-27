# mdBook specific markdown

## Including files

With the following syntax, you can include files into your book:

```hbs
\{{#include file.rs}}
```

The path to the file has to be relative from the current source file.

Usually, this command is used for including code snippets and examples. In this case, oftens one would include a specific part of the file e.g. which only contains the relevant lines for the example. We support four different modes of partial includes:

```hbs
\{{#include file.rs:2}}
\{{#include file.rs::10}}
\{{#include file.rs:2:}}
\{{#include file.rs:2:10}}
```

The first command only includes the second line from file `file.rs`. The second command includes all lines up to line 10, i.e. the lines from 11 till the end of the file are omitted. The third command includes all lines from line 2, i.e. the first line is omitted. The last command includes the excerpt of `file.rs` consisting of lines 2 to 10.
