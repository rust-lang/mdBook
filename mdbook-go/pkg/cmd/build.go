package cmd

import (
	"path/filepath"

	"mdbook-go/internal/runner"

	"github.com/spf13/cobra"
)

// NewBuildCommand implements the `doclens build` subcommand: render the
// book into its build directory. It mirrors src/cmd/build.rs.
func NewBuildCommand() *cobra.Command {
	var dir, dest string
	var openAfter bool

	cmd := &cobra.Command{
		Use:   "build",
		Short: "build a book",
		RunE: func(cmd *cobra.Command, args []string) error {
			return runBuild(dir, dest, openAfter)
		},
	}

	cmd.Flags().StringVar(&dir, "dir", ".", "book root")
	cmd.Flags().StringVar(&dest, "dest-dir", "", "output directory (overrides doclens.yaml)")
	cmd.Flags().BoolVar(&openAfter, "open", false, "open the rendered book in the default browser after building")

	return cmd
}

func runBuild(dir, dest string, openAfter bool) error {
	m, err := runner.Load(dir)
	if err != nil {
		return err
	}
	if dest != "" {
		m.Config.Build.BuildDir = dest
	}
	if err := m.Build(); err != nil {
		return err
	}
	if openAfter {
		// The Rust version opens <build_dir>/html/index.html. Our default
		// build directory is the user's build-dir, where the index lives
		// directly as index.html (M2 outputs it alongside the per-chapter
		// pages, 404.html, etc.).
		index := filepath.Join(m.BuildDir(), "index.html")
		if err := open(index); err != nil {
			return err
		}
	}
	return nil
}
