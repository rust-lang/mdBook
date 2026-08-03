package driver

import (
	"fmt"
	"os"
	"os/exec"
	"runtime"
)

// Open launches the OS default handler for path in the background. It is a
// portable equivalent to Rust's `opener::open` used by
// crates/mdbook/src/main.rs::open.
//
// On macOS the command is `open`, on Windows `cmd /c start ""`, on Linux
// and other Unixes `xdg-open`. The process is started detached (its
// stdout/stderr go to the void) and Open returns as soon as the spawn
// succeeds; we don't wait for the browser to close. This matches the
// "fire and forget" semantics of the Rust implementation.
func Open(path string) error {
	if path == "" {
		return fmt.Errorf("open: empty path")
	}
	if _, err := os.Stat(path); err != nil {
		return fmt.Errorf("open: %w", err)
	}
	var cmd *exec.Cmd
	switch runtime.GOOS {
	case "darwin":
		cmd = exec.Command("open", path)
	case "windows":
		// The empty "" is a placeholder title that start.exe requires
		// when the first quoted argument is a path rather than a title.
		cmd = exec.Command("cmd", "/c", "start", "", path)
	default:
		cmd = exec.Command("xdg-open", path)
	}
	// Detach from the parent so closing the mdbook terminal doesn't kill
	// the browser window before the user has read it.
	cmd.Stdin = nil
	cmd.Stdout = nil
	cmd.Stderr = nil
	if err := cmd.Start(); err != nil {
		return fmt.Errorf("open %q: %w", path, err)
	}
	// Reap asynchronously to avoid zombies on Unix. The handle is held
	// only until Wait returns; the user is never blocked.
	go func() { _ = cmd.Wait() }()
	return nil
}
