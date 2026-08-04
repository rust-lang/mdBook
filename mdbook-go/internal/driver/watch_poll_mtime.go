package driver

import (
	"os"
	"time"
)

// mtime returns the file's modification time as reported by os.FileInfo.
// This mirrors src/cmd/watch/poller.rs::Watcher::scan, which uses
// `meta.modified().unwrap_or(SystemTime::UNIX_EPOCH)` directly with no
// platform-specific branch.
//
// Earlier revisions of this file tried to fall back to
// syscall.Stat_t.Ctim when ModTime() returned zero — a paranoid measure
// that did not actually compile on macOS (the field is named Ctimespec
// on Darwin, Ctim on Linux) and would have produced spurious rebuilds
// anyway, since ctime tracks metadata changes (chmod / rename / owner)
// rather than content. Reverting to the simpler "ModTime or zero"
// semantic keeps the Go watcher bit-for-bit equivalent to the Rust one.
func mtime(info os.FileInfo) time.Time {
	return info.ModTime()
}
