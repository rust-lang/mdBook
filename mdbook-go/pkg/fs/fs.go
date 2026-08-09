package fs

import (
	"io"
	"os"
	"path/filepath"
	"strings"
)

// PathToRoot returns just enough `../` segments to walk from the directory
// containing path back to the book root. It is a port of path_to_root in
// crates/mdbook-core/src/utils/fs.rs, so "intro.md" yields "", "a/b.md" yields
// "../" and "a/b/c.md" yields "../../".
func PathToRoot(path string) string {
	path = strings.ReplaceAll(path, "\\", "/")
	idx := strings.LastIndex(path, "/")
	if idx < 0 {
		return ""
	}
	var b strings.Builder
	for _, part := range strings.Split(path[:idx], "/") {
		switch part {
		case "", ".", "..":
			// Rust only counts Component::Normal; the rest is skipped.
		default:
			b.WriteString("../")
		}
	}
	return b.String()
}

// ToURLPath converts an on-disk relative path into a URL path.
func ToURLPath(path string) string {
	return strings.ReplaceAll(path, string(filepath.Separator), "/")
}

// NormalizePath resolves `.` and `..` lexically without touching the
// filesystem, matching normalize_path in crates/mdbook-html/src/utils.rs.
func NormalizePath(path string) string {
	rooted := strings.HasPrefix(path, "/")
	var out []string
	for _, part := range strings.Split(strings.ReplaceAll(path, "\\", "/"), "/") {
		switch part {
		case "", ".":
		case "..":
			if len(out) > 0 && out[len(out)-1] != ".." {
				out = out[:len(out)-1]
			} else if !rooted {
				out = append(out, "..")
			}
		default:
			out = append(out, part)
		}
	}
	joined := strings.Join(out, "/")
	if rooted {
		return "/" + joined
	}
	return joined
}

// RemoveDirContent empties dir without removing dir itself.
func RemoveDirContent(dir string) error {
	entries, err := os.ReadDir(dir)
	if err != nil {
		return err
	}
	for _, entry := range entries {
		if err := os.RemoveAll(filepath.Join(dir, entry.Name())); err != nil {
			return err
		}
	}
	return nil
}

// CopyFilesExceptExt recursively copies from into to, skipping files whose
// extension is listed in extBlacklist (without the leading dot) and skipping
// avoidDir. It is a port of copy_files_except_ext in
// crates/mdbook-core/src/utils/fs.rs.
func CopyFilesExceptExt(from, to string, recursive bool, avoidDir string, extBlacklist []string) error {
	if from == to {
		return nil
	}
	entries, err := os.ReadDir(from)
	if err != nil {
		return err
	}
	for _, entry := range entries {
		src := filepath.Join(from, entry.Name())
		dst := filepath.Join(to, entry.Name())
		info, err := entry.Info()
		if err != nil {
			return err
		}
		switch {
		case info.IsDir() && recursive:
			if src == to || (avoidDir != "" && src == avoidDir) {
				continue
			}
			if _, err := os.Stat(dst); os.IsNotExist(err) {
				if err := os.Mkdir(dst, 0o755); err != nil {
					return err
				}
			}
			if err := CopyFilesExceptExt(src, dst, true, avoidDir, extBlacklist); err != nil {
				return err
			}
		case info.Mode().IsRegular():
			if ext := strings.TrimPrefix(filepath.Ext(entry.Name()), "."); ext != "" {
				if contains(extBlacklist, ext) {
					continue
				}
			}
			if err := CopyFile(src, dst); err != nil {
				return err
			}
		}
	}
	return nil
}

// CopyFile copies a single file, creating parent directories as needed.
func CopyFile(src, dst string) error {
	if err := os.MkdirAll(filepath.Dir(dst), 0o755); err != nil {
		return err
	}
	in, err := os.Open(src)
	if err != nil {
		return err
	}
	defer in.Close()
	info, err := in.Stat()
	if err != nil {
		return err
	}
	out, err := os.OpenFile(dst, os.O_WRONLY|os.O_CREATE|os.O_TRUNC, info.Mode().Perm())
	if err != nil {
		return err
	}
	if _, err := io.Copy(out, in); err != nil {
		out.Close()
		return err
	}
	return out.Close()
}

// WriteFile writes data to path, creating parent directories as needed.
func WriteFile(path string, data []byte) error {
	if err := os.MkdirAll(filepath.Dir(path), 0o755); err != nil {
		return err
	}
	return os.WriteFile(path, data, 0o644)
}

func contains(list []string, value string) bool {
	for _, item := range list {
		if item == value {
			return true
		}
	}
	return false
}
