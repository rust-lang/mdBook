#### --watcher

There are different backends used to determine when a file has changed.

* `poll` (default) --- Checks for file modifications by scanning the filesystem every second.
* `native` --- Uses the native operating system facilities to receive notifications when files change.
  This can have less constant overhead, but may not be as reliable as the `poll` based watcher.
