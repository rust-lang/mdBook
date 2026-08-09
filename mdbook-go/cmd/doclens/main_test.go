package main

import (
	"errors"
	"fmt"
	"testing"
)

// TestFormatErrorChain verifies that formatError produces the same
// multi-line "Caused by:" chain that Rust mdBook's `utils::log_backtrace`
// emits in src/main.rs::main. The chain depth here matches a realistic
// error path through the load pipeline (Load → read doclens.yaml → open file).
func TestFormatErrorChain(t *testing.T) {
	// Build a 3-deep error chain the way fmt.Errorf("...: %w", inner) does.
	inner := errors.New("no such file or directory")
	mid := fmt.Errorf("open /tmp/foo/doclens.yaml: %w", inner)
	top := fmt.Errorf("read book config: %w", mid)

	got := formatError(top)
	want := "read book config: open /tmp/foo/doclens.yaml: no such file or directory\n" +
		"\tCaused by: open /tmp/foo/doclens.yaml: no such file or directory\n" +
		"\tCaused by: no such file or directory\n"
	if got != want {
		t.Errorf("formatError mismatch:\n got: %q\nwant: %q", got, want)
	}
}

// TestFormatErrorNil verifies that formatError on a nil error returns
// the empty string. This guards against future callers mistakenly
// forwarding nil through main and getting an unexpected stderr write.
func TestFormatErrorNil(t *testing.T) {
	if got := formatError(nil); got != "" {
		t.Errorf("formatError(nil) = %q, want \"\"", got)
	}
}

// TestFormatErrorSingle is the trivial case: an error with no wrapped
// cause should print just the top-level message with no "Caused by:" line.
func TestFormatErrorSingle(t *testing.T) {
	got := formatError(errors.New("boom"))
	want := "boom\n"
	if got != want {
		t.Errorf("formatError(single) = %q, want %q", got, want)
	}
}
