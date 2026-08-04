package cli

import (
	"errors"
	"fmt"
	"testing"
)

// TestFormatErrorChain verifies that FormatError produces the same
// multi-line "Caused by:" chain that Rust mdBook's `utils::log_backtrace`
// emits in src/main.rs::main. The chain depth here matches a realistic
// error path through the load pipeline (Load → read book.toml → open file).
func TestFormatErrorChain(t *testing.T) {
	// Build a 3-deep error chain the way fmt.Errorf("...: %w", inner) does.
	inner := errors.New("no such file or directory")
	mid := fmt.Errorf("open /tmp/foo/book.toml: %w", inner)
	top := fmt.Errorf("read book config: %w", mid)

	got := FormatError(top)
	want := "read book config: open /tmp/foo/book.toml: no such file or directory\n" +
		"\tCaused by: open /tmp/foo/book.toml: no such file or directory\n" +
		"\tCaused by: no such file or directory\n"
	if got != want {
		t.Errorf("FormatError mismatch:\n got: %q\nwant: %q", got, want)
	}
}

// TestFormatErrorNil verifies that FormatError on a nil error returns
// the empty string. This guards against future callers mistakenly
// forwarding nil through HandleError and getting an unexpected stderr
// write.
func TestFormatErrorNil(t *testing.T) {
	if got := FormatError(nil); got != "" {
		t.Errorf("FormatError(nil) = %q, want \"\"", got)
	}
}

// TestFormatErrorSingle is the trivial case: an error with no wrapped
// cause should print just the top-level message with no "Caused by:" line.
func TestFormatErrorSingle(t *testing.T) {
	got := FormatError(errors.New("boom"))
	want := "boom\n"
	if got != want {
		t.Errorf("FormatError(single) = %q, want %q", got, want)
	}
}