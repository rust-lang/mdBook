// Package cmd wires up the doclens CLI: the root command plus every
// subcommand. Single-file commands are flattened into this package (one
// file per command, mirroring src/cmd/*.rs), and shared command-layer
// helpers like open() (main.rs::open) live here too so they can be
// called directly. The watch engine's PollWatcher implementation lives
// in the pkg/cmd/watch subpackage (mirroring src/cmd/watch/poller.rs).
package cmd

import (
	"mdbook-go/pkg/cmd/completion"

	"github.com/spf13/cobra"
)

func New() *cobra.Command {
	cmd := &cobra.Command{
		Use:           "doclens",
		Short:         "A lightweight local markdown documentation preview tool",
		Long:          "A lightweight local markdown documentation preview tool",
		SilenceErrors: true,
		SilenceUsage:  true,
		Version:       Version,
	}

	cmd.AddCommand(NewBuildCommand())
	cmd.AddCommand(NewCleanCommand())
	cmd.AddCommand(completion.NewCommand())
	cmd.AddCommand(NewCreateCommand())
	cmd.AddCommand(NewServeCommand())
	cmd.AddCommand(NewVersionCommand())
	cmd.AddCommand(NewWatchCommand())

	return cmd
}
