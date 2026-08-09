package cmd

import (
	"fmt"
	"io/fs"
	"os"
	"path/filepath"

	"mdbook-go/internal/runner"

	"github.com/spf13/cobra"
)

// NewCleanCommand implements the `doclens clean` subcommand: remove the
// book's build directory. It mirrors src/cmd/clean.rs, which is also a
// single file holding both the command and the Clean summary type.
func NewCleanCommand() *cobra.Command {
	var dir, dest string

	cmd := &cobra.Command{
		Use:   "clean",
		Short: "remove the build directory",
		RunE: func(cmd *cobra.Command, args []string) error {
			return runClean(dir, dest)
		},
	}

	cmd.Flags().StringVar(&dir, "dir", ".", "book root")
	cmd.Flags().StringVar(&dest, "dest-dir", "", "directory to remove (overrides doclens.yaml build-dir)")

	return cmd
}

// runClean removes the book's build directory. It loads the book first to
// resolve the configured build-dir, but if dest is non-empty the override
// is honoured before Load is even called (matching Rust's `--dest-dir`
// semantics in src/cmd/clean.rs).
func runClean(dir, dest string) error {
	if dest != "" {
		// Override path: no need to load the book at all.
		c, err := removeBuildDir(dest)
		if err != nil {
			return err
		}
		fmt.Println(c)
		return nil
	}
	m, err := runner.Load(dir)
	if err != nil {
		return err
	}
	c, err := removeBuildDir(m.BuildDir())
	if err != nil {
		return err
	}
	fmt.Println(c)
	return nil
}

// Clean describes the result of removing a build directory. It mirrors the
// output shape of crates/mdbook/src/cmd/clean.rs so the CLI can render the
// same summary line in either implementation.
type Clean struct {
	NumFilesRemoved   uint64
	NumDirsRemoved    uint64
	TotalBytesRemoved uint64
}

// removeBuildDir walks dir breadth-first to count files/dirs/bytes, then
// removes the whole tree. A non-existent dir returns zero stats without an
// error, matching the Rust `Clean::new` behaviour.
func removeBuildDir(dir string) (*Clean, error) {
	info, err := os.Stat(dir)
	if err != nil {
		if os.IsNotExist(err) {
			return &Clean{}, nil
		}
		return nil, err
	}
	if !info.IsDir() {
		// `dir` is a single file path; remove it and report as one file.
		c := &Clean{NumFilesRemoved: 1, TotalBytesRemoved: uint64(info.Size())}
		if err := os.Remove(dir); err != nil {
			return nil, err
		}
		return c, nil
	}

	var c Clean
	walkErr := filepath.WalkDir(dir, func(path string, d fs.DirEntry, walkErr error) error {
		if walkErr != nil {
			return walkErr
		}
		info, err := d.Info()
		if err != nil {
			return err
		}
		// Note: the byte tally mirrors Rust's behaviour: it sums exact byte
		// sizes, not block sizes, and may over-count when files are
		// hard-linked. That trade-off is acceptable for the user-facing
		// summary.
		c.TotalBytesRemoved += uint64(info.Size())
		if d.IsDir() {
			c.NumDirsRemoved++
		} else {
			c.NumFilesRemoved++
		}
		return nil
	})
	if walkErr != nil {
		return nil, walkErr
	}

	if err := os.RemoveAll(dir); err != nil {
		return nil, fmt.Errorf("remove build directory %q: %w", dir, err)
	}
	return &c, nil
}

// String renders a Clean summary in the same format as the Rust CLI:
//   - 0 files / 1 directory / N directories / 1 file / N files
//   - ", NB total" or ", <value><unit> total" depending on byte size
func (c *Clean) String() string {
	var files string
	switch {
	case c.NumFilesRemoved == 0 && c.NumDirsRemoved == 0:
		files = "0 files"
	case c.NumFilesRemoved == 0 && c.NumDirsRemoved == 1:
		files = "1 directory"
	case c.NumFilesRemoved == 0:
		files = fmt.Sprintf("%d directories", c.NumDirsRemoved)
	case c.NumFilesRemoved == 1:
		files = "1 file"
	default:
		files = fmt.Sprintf("%d files", c.NumFilesRemoved)
	}
	if c.TotalBytesRemoved == 0 {
		return "Removed " + files
	}
	if c.TotalBytesRemoved < 1024 {
		return fmt.Sprintf("Removed %s, %dB total", files, c.TotalBytesRemoved)
	}
	val, unit := humanReadableBytes(c.TotalBytesRemoved)
	return fmt.Sprintf("Removed %s, %.2f%s total", files, val, unit)
}

// humanReadableBytes picks the largest SI binary unit that keeps the value
// below 1024. The list and threshold match the Rust implementation in
// crates/mdbook/src/cmd/clean.rs::human_readable_bytes.
func humanReadableBytes(bytes uint64) (float32, string) {
	units := []string{"B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB"}
	b := float32(bytes)
	i := 0
	for b >= 1024 && i < len(units)-1 {
		b /= 1024
		i++
	}
	return b, units[i]
}
