//go:build windows

package driver

import (
	"os"
	"time"
)

// mtime returns the file's modification time. On Windows
// os.FileInfo.ModTime() is sourced from syscall.Win32FileAttributeData's
// LastWriteTime, which is already stable across the filesystems we
// support, so no ctime-style fallback is needed.
func mtime(info os.FileInfo) time.Time {
	return info.ModTime()
}
