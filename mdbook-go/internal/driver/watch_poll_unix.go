//go:build !windows

package driver

import (
	"os"
	"syscall"
	"time"
)

// mtime returns a stable modification time. On filesystems where the
// underlying stat does not provide a usable mtime we fall back to the
// ctime (via syscall.Stat_t) so the watcher still has a value to diff
// against; without this, every scan would flag every file as changed.
func mtime(info os.FileInfo) time.Time {
	if t := info.ModTime(); !t.IsZero() {
		return t
	}
	if stat, ok := info.Sys().(*syscall.Stat_t); ok {
		// Ctime is the metadata change time on Unix, but it's the best
		// monotonic value we have when ModTime is zero. Resolution is
		// filesystem-dependent; this branch only fires on the few
		// platforms where ModTime() returns the zero time.
		return time.Unix(stat.Ctim.Sec, stat.Ctim.Nsec)
	}
	return time.Time{}
}
