package cmd

import (
	"fmt"
	"os"
	"runtime"

	"github.com/spf13/cobra"
)

// Version is the doclens CLI version, injected at build time when wanted.
var Version = "0.1.0"

func GetVersion() string {
	return "doclens v" + Version + " " + runtime.Version() + " " + runtime.GOOS + "/" + runtime.GOARCH
}

// NewVersionCommand implements the `doclens version` subcommand.
func NewVersionCommand() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "version",
		Short: "Prints the doclens CLI version",
		Long:  "Prints the doclens CLI version",
		RunE: func(cmd *cobra.Command, args []string) error {
			fmt.Fprintln(os.Stdout, GetVersion())
			return nil
		},
	}
	return cmd
}
