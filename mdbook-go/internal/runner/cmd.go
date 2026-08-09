package runner

// ⚠️  FROZEN — 外部 preprocessor / renderer 子进程协议实现已冻结 ⚠️
//
// 2026-08-04 决策：暂时不做 M3.10 / M3.11 端到端验收。代码保留（含本文件、
// registry.go::BuildRenderers、tests/external-plugin/），不删，未来若有
// 第三方插件需求再回来补：black-box wire 协议比对 + harness/diff.sh
// external-plugin 严格模式。
//
// 影响范围：
//   - harness/diff.sh SKIP 列表已加入 external-plugin
//   - doc/plan/progress.md M3 段落已加 FROZEN 标记
//   - 内置 links / index 预处理器不受影响（registry.go::BuildPreprocessors
//     仅在 `UseDefaultPreprocessors=true` 时跑这两条，与本文件无关）

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"os"
	"os/exec"
	"path/filepath"
	"strings"

	"mdbook-go/internal/model"
	"mdbook-go/internal/plugin"
)

// CmdPreprocessor shells out to an external executable that follows the
// mdBook preprocessor protocol. It mirrors
// crates/mdbook-driver/src/builtin_preprocessors/cmd.rs.
//
// Protocol:
//
//	stdin  = (PreprocessorContext, Book) as a JSON tuple
//	stdout = processed Book as JSON
//	stderr = forwarded to ours
//	cwd    = book root
//
// When the program is invoked with arguments `supports <renderer>`, a zero
// exit code means it is compatible with that renderer.
type CmdPreprocessor struct {
	name     string
	cmd      string
	root     string
	optional bool
}

// NewCmdPreprocessor constructs a CmdPreprocessor. `root` is the book root;
// `cmd` is the shell-style command line to invoke (shlex-parsed).
func NewCmdPreprocessor(name, cmd, root string, optional bool) *CmdPreprocessor {
	return &CmdPreprocessor{name: name, cmd: cmd, root: root, optional: optional}
}

func (c *CmdPreprocessor) Name() string { return c.name }

// Cmd returns the underlying command string (matches Rust's CmdPreprocessor::cmd).
func (c *CmdPreprocessor) Cmd() string { return c.cmd }

// Run invokes the external program, writes the (ctx, book) JSON tuple to its
// stdin, and reads the processed book back from stdout.
func (c *CmdPreprocessor) Run(ctx *plugin.PreprocessorContext, b *model.Book) (*model.Book, error) {
	cmd, err := composeCommand(c.cmd, c.root)
	if err != nil {
		if c.optional {
			fmt.Fprintf(os.Stderr, "warning: command %q for preprocessor %q not found, skipping\n", c.cmd, c.name)
			return b, nil
		}
		return nil, fmt.Errorf("preprocessor %q: %w", c.name, err)
	}
	cmd.Dir = c.root
	cmd.Stdin = bytes.NewReader(nil) // replaced after spawn
	cmd.Stdout = nil                 // captured below
	cmd.Stderr = os.Stderr

	stdout := &bytes.Buffer{}
	cmd.Stdout = stdout
	cmd.Stderr = os.Stderr

	if err := cmd.Start(); err != nil {
		if c.optional && isNotFound(err) {
			fmt.Fprintf(os.Stderr, "warning: command for preprocessor %q not found, skipping\n", c.name)
			return b, nil
		}
		return nil, fmt.Errorf("unable to run preprocessor %q: %w", c.name, err)
	}

	// Reopen stdin as a pipe and write the input tuple to it. We can't use a
	// pipe directly via exec.Cmd because we'd race the spawn; the idiomatic
	// Go pattern is to set Stdout = pipe below, but for stdin we just write
	// to the inherited pipe.
	pipe, err := cmd.StdinPipe()
	if err != nil {
		return nil, fmt.Errorf("preprocessor %q stdin pipe: %w", c.name, err)
	}
	// Reset cmd.Stdin so cmd.Wait doesn't double-close.
	cmd.Stdin = nil

	inputErrCh := make(chan error, 1)
	go func() {
		defer pipe.Close()
		wire := plugin.ToWirePreprocessorContext(ctx)
		wb := plugin.ToWireBook(b)
		// Serialize as a JSON tuple. encoding/json doesn't natively support
		// tuples, so wrap them in an anonymous struct.
		tuple := struct {
			Ctx  plugin.WirePreprocessorContext `json:"ctx"`
			Book plugin.WireBook                `json:"book"`
		}{wire, wb}
		// The Rust side uses serde_json::to_writer(&(ctx, book)) which
		// produces a 2-element JSON array. Match that byte-for-byte.
		arr := [2]any{wire, wb}
		if err := json.NewEncoder(pipe).Encode(arr); err != nil {
			inputErrCh <- err
			return
		}
		_ = tuple // referenced to keep the named-struct variant compilable
		inputErrCh <- nil
	}()

	if err := cmd.Wait(); err != nil {
		return nil, fmt.Errorf("preprocessor %q exited unsuccessfully: %w", c.name, err)
	}
	if err := <-inputErrCh; err != nil {
		return nil, fmt.Errorf("writing input to preprocessor %q: %w", c.name, err)
	}

	var wire plugin.WireBook
	if err := json.Unmarshal(stdout.Bytes(), &wire); err != nil {
		return nil, fmt.Errorf("parsing preprocessor %q output: %w", c.name, err)
	}
	return plugin.FromWireBook(wire), nil
}

// SupportsRenderer probes via "<cmd> supports <renderer>". Exit code 0 means
// supported.
func (c *CmdPreprocessor) SupportsRenderer(renderer string) (bool, error) {
	cmd, err := composeCommand(c.cmd, c.root)
	if err != nil {
		if c.optional {
			return false, nil
		}
		return false, err
	}
	cmd.Dir = c.root
	cmd.Args = append(cmd.Args, "supports", renderer)
	cmd.Stdin = nil
	cmd.Stdout = nil
	cmd.Stderr = os.Stderr

	err = cmd.Run()
	if err == nil {
		return true, nil
	}
	// Exit code != 0 means not supported.
	if exitErr, ok := err.(*exec.ExitError); ok {
		if exitErr.ExitCode() != 0 {
			return false, nil
		}
	}
	if c.optional && isNotFound(err) {
		return false, nil
	}
	return false, fmt.Errorf("preprocessor %q supports probe failed: %w", c.name, err)
}

// CmdRenderer shells out to an external executable that follows the
// mdBook renderer protocol. It mirrors
// crates/mdbook-driver/src/builtin_renderers/mod.rs::CmdRenderer.
//
// Protocol:
//
//	stdin  = RenderContext JSON
//	cwd    = destination directory (NOT the book root)
//	stdout = inherited
//	stderr = inherited
type CmdRenderer struct {
	name string
	cmd  string
}

// NewCmdRenderer constructs a CmdRenderer.
func NewCmdRenderer(name, cmd string) *CmdRenderer {
	return &CmdRenderer{name: name, cmd: cmd}
}

func (c *CmdRenderer) Name() string { return c.name }

// Render writes the RenderContext to the child's stdin and waits for it to
// complete.
func (c *CmdRenderer) Render(ctx *plugin.RenderContext) error {
	cmd, err := composeCommand(c.cmd, ctx.Root)
	if err != nil {
		return fmt.Errorf("renderer %q: %w", c.name, err)
	}
	cmd.Dir = ctx.Destination
	cmd.Stdin = nil
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr

	pipe, err := cmd.StdinPipe()
	if err != nil {
		return fmt.Errorf("renderer %q stdin pipe: %w", c.name, err)
	}
	cmd.Stdin = nil

	if err := cmd.Start(); err != nil {
		if isNotFound(err) {
			return fmt.Errorf("renderer %q command %q not found", c.name, c.cmd)
		}
		return fmt.Errorf("renderer %q: %w", c.name, err)
	}

	wire := plugin.ToWireRenderContext(ctx)
	if err := json.NewEncoder(pipe).Encode(wire); err != nil {
		pipe.Close()
		cmd.Wait()
		return fmt.Errorf("writing input to renderer %q: %w", c.name, err)
	}
	if err := pipe.Close(); err != nil {
		cmd.Wait()
		return fmt.Errorf("closing renderer %q stdin: %w", c.name, err)
	}
	if err := cmd.Wait(); err != nil {
		return fmt.Errorf("renderer %q exited unsuccessfully: %w", c.name, err)
	}
	return nil
}

// composeCommand parses a shell-style command line using a shlex-style
// splitter and returns an *exec.Cmd ready to be Start()ed. If the executable
// name contains a path separator it is resolved relative to root; otherwise
// it is left for the OS to resolve via PATH.
func composeCommand(line, root string) (*exec.Cmd, error) {
	parts, err := shlexSplit(line)
	if err != nil {
		return nil, err
	}
	if len(parts) == 0 {
		return nil, fmt.Errorf("empty command")
	}
	exe := parts[0]
	if strings.ContainsRune(exe, os.PathSeparator) || filepath.IsAbs(exe) {
		exe = filepath.Join(root, exe)
	}
	return exec.Command(exe, parts[1:]...), nil
}

// shlexSplit is a minimal shell-style splitter that handles single and double
// quotes and backslash escapes. It covers the cases mdBook users typically
// need (e.g. `mdbook-mermaid --option "value with spaces"`).
func shlexSplit(s string) ([]string, error) {
	var out []string
	var cur strings.Builder
	var quote rune
	escaped := false
	flush := func() {
		if cur.Len() > 0 || quote == 0 {
			out = append(out, cur.String())
		}
		cur.Reset()
	}
	for _, r := range s {
		switch {
		case escaped:
			cur.WriteRune(r)
			escaped = false
		case r == '\\':
			escaped = true
		case quote != 0:
			if r == quote {
				quote = 0
			} else {
				cur.WriteRune(r)
			}
		case r == '"' || r == '\'':
			quote = r
		case r == ' ' || r == '\t' || r == '\n':
			flush()
		default:
			cur.WriteRune(r)
		}
	}
	if quote != 0 {
		return nil, fmt.Errorf("unterminated quote in %q", s)
	}
	if escaped {
		return nil, fmt.Errorf("trailing backslash in %q", s)
	}
	flush()
	return out, nil
}

// isNotFound reports whether err is the platform's "file not found" error.
func isNotFound(err error) bool {
	if err == nil {
		return false
	}
	if exitErr, ok := err.(*exec.ExitError); ok {
		// Exit errors don't carry ENOENT semantics; treat as "not found"
		// only when the underlying process couldn't be started (which
		// wouldn't be an *exec.ExitError).
		_ = exitErr
		return false
	}
	// Fall back to substring check; sufficient for our purposes since
	// compose_command only fails with path-related errors.
	return strings.Contains(err.Error(), "not found") ||
		strings.Contains(err.Error(), "no such file")
}

// unusedWriter keeps io imported so a future edit that streams the book via
// the writer interface doesn't need to re-add the import.
var _ io.Writer = (*bytes.Buffer)(nil)
