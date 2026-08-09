package cmd

import (
	"mdbook-go/internal/runner"

	"github.com/spf13/cobra"
)

// NewCreateCommand implements the `doclens create` subcommand: create a
// new book skeleton. It mirrors src/cmd/init.rs.
func NewCreateCommand() *cobra.Command {
	var dir string
	var theme, force bool
	var title string

	cmd := &cobra.Command{
		Use:   "create",
		Short: "create a new book skeleton",
		RunE: func(cmd *cobra.Command, args []string) error {
			return runner.Init(dir, runner.InitOptions{
				Title:  title,
				Theme:  theme,
				Force:  force,
			})
		},
	}

	cmd.Flags().StringVar(&dir, "dir", ".", "book root")
	cmd.Flags().BoolVar(&theme, "theme", false, "copy default theme into <dir>/theme")
	cmd.Flags().BoolVar(&force, "force", false, "skip confirmation prompts (no prompts exist yet; accepted for parity with Rust)")
	cmd.Flags().StringVar(&title, "title", "", "book title (default \"My Book\" if empty)")

	return cmd
}
