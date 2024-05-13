#### `--watcher`

There are different backends used to determine when a file has changed.

* `poll` (default) --- Checks for file modifications by scanning the filesystem every second.
* `native` --- Uses the native operating system facilities to receive notifications when files change.
  This can have less constant overhead, but may not be as reliable as the `poll` based watcher. See these issues for more information: [#383](https://github.com/rust-lang/mdBook/issues/383) [#1441](https://github.com/rust-lang/mdBook/issues/1441) [#1707](https://github.com/rust-lang/mdBook/issues/1707) [#2035](https://github.com/rust-lang/mdBook/issues/2035) [#2102](https://github.com/rust-lang/mdBook/issues/2102)
