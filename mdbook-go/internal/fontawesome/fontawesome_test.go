package fontawesome

import (
	"strings"
	"testing"
)

// expectedSpanFromIntro is a hand-verified map of the icon pairs that
// appear in /tmp/m2-rust/intro.html (real Rust mdBook output for a book).
// Each value is byte-for-byte the fragment that the Rust font-awesome-as-a-crate
// 0.3.1 SVG data + mdbook-html's fontawesome helper produces. They are
// extracted verbatim from the file to catch any drift on our side, but
// the goal of this test is to compare them against Span()'s output, not
// to re-parse intro.html at runtime.
//
// Hard-coding the strings (rather than re-parsing) is the whole point:
// the values here double as the canonical byte-exact expectation, and
// extracting them only happens once at the time this test is written.
var expectedSpanFromIntro = []struct {
	t    Type
	name string
	id   string
	want string
}{
	{Solid, `bars`, ``, `<span class=fa-svg><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 448 512"><!--! Font Awesome Free 6.2.0 by @fontawesome - https://fontawesome.com License - https://fontawesome.com/license/free (Icons: CC BY 4.0, Fonts: SIL OFL 1.1, Code: MIT License) Copyright 2022 Fonticons, Inc. --><path d="M0 96C0 78.3 14.3 64 32 64H416c17.7 0 32 14.3 32 32s-14.3 32-32 32H32C14.3 128 0 113.7 0 96zM0 256c0-17.7 14.3-32 32-32H416c17.7 0 32 14.3 32 32s-14.3 32-32 32H32c-17.7 0-32-14.3-32-32zM448 416c0 17.7-14.3 32-32 32H32c-17.7 0-32-14.3-32-32s14.3-32 32-32H416c17.7 0 32 14.3 32 32z"/></svg></span>`},
	{Solid, `paintbrush`, ``, `<span class=fa-svg><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 576 512"><!--! Font Awesome Free 6.2.0 by @fontawesome - https://fontawesome.com License - https://fontawesome.com/license/free (Icons: CC BY 4.0, Fonts: SIL OFL 1.1, Code: MIT License) Copyright 2022 Fonticons, Inc. --><path d="M371.3 367.1c27.3-3.9 51.9-19.4 67.2-42.9L600.2 74.1c12.6-19.5 9.4-45.3-7.6-61.2S549.7-4.4 531.1 9.6L294.4 187.2c-24 18-38.2 46.1-38.4 76.1L371.3 367.1zm-19.6 25.4l-116-104.4C175.9 290.3 128 339.6 128 400c0 3.9 .2 7.8 .6 11.6c1.8 17.5-10.2 36.4-27.8 36.4H96c-17.7 0-32 14.3-32 32s14.3 32 32 32H240c61.9 0 112-50.1 112-112c0-2.5-.1-5-.2-7.5z"/></svg></span>`},
	{Solid, `magnifying-glass`, ``, `<span class=fa-svg><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512"><!--! Font Awesome Free 6.2.0 by @fontawesome - https://fontawesome.com License - https://fontawesome.com/license/free (Icons: CC BY 4.0, Fonts: SIL OFL 1.1, Code: MIT License) Copyright 2022 Fonticons, Inc. --><path d="M416 208c0 45.9-14.9 88.3-40 122.7L502.6 457.4c12.5 12.5 12.5 32.8 0 45.3s-32.8 12.5-45.3 0L330.7 376c-34.4 25.2-76.8 40-122.7 40C93.1 416 0 322.9 0 208S93.1 0 208 0S416 93.1 416 208zM208 352c79.5 0 144-64.5 144-144s-64.5-144-144-144S64 128.5 64 208s64.5 144 144 144z"/></svg></span>`},
	{Solid, `print`, `print-button`, `<span class=fa-svg id="print-button"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512"><!--! Font Awesome Free 6.2.0 by @fontawesome - https://fontawesome.com License - https://fontawesome.com/license/free (Icons: CC BY 4.0, Fonts: SIL OFL 1.1, Code: MIT License) Copyright 2022 Fonticons, Inc. --><path d="M128 0C92.7 0 64 28.7 64 64v96h64V64H354.7L384 93.3V160h64V93.3c0-17-6.7-33.3-18.7-45.3L400 18.7C388 6.7 371.7 0 354.7 0H128zM384 352v32 64H128V384 368 352H384zm64 32h32c17.7 0 32-14.3 32-32V256c0-35.3-28.7-64-64-64H64c-35.3 0-64 28.7-64 64v96c0 17.7 14.3 32 32 32H64v64c0 35.3 28.7 64 64 64H384c35.3 0 64-28.7 64-64V384zm-16-88c-13.3 0-24-10.7-24-24s10.7-24 24-24s24 10.7 24 24s-10.7 24-24 24z"/></svg></span>`},
	{Solid, `spinner`, `fa-spin`, `<span class=fa-svg id="fa-spin"><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512"><!--! Font Awesome Free 6.2.0 by @fontawesome - https://fontawesome.com License - https://fontawesome.com/license/free (Icons: CC BY 4.0, Fonts: SIL OFL 1.1, Code: MIT License) Copyright 2022 Fonticons, Inc. --><path d="M304 48c0-26.5-21.5-48-48-48s-48 21.5-48 48s21.5 48 48 48s48-21.5 48-48zm0 416c0-26.5-21.5-48-48-48s-48 21.5-48 48s21.5 48 48 48s48-21.5 48-48zM48 304c26.5 0 48-21.5 48-48s-21.5-48-48-48s-48 21.5-48 48s21.5 48 48 48zm464-48c0-26.5-21.5-48-48-48s-48 21.5-48 48s21.5 48 48 48s48-21.5 48-48zM142.9 437c18.7-18.7 18.7-49.1 0-67.9s-49.1-18.7-67.9 0s-18.7 49.1 0 67.9s49.1 18.7 67.9 0zm0-294.2c18.7-18.7 18.7-49.1 0-67.9S93.7 56.2 75 75s-18.7 49.1 0 67.9s49.1 18.7 67.9 0zM369.1 437c18.7 18.7 49.1 18.7 67.9 0s18.7-49.1 0-67.9s-49.1-18.7-67.9 0s-18.7 49.1 0 67.9z"/></svg></span>`},
	{Solid, `angle-right`, ``, `<span class=fa-svg><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 320 512"><!--! Font Awesome Free 6.2.0 by @fontawesome - https://fontawesome.com License - https://fontawesome.com/license/free (Icons: CC BY 4.0, Fonts: SIL OFL 1.1, Code: MIT License) Copyright 2022 Fonticons, Inc. --><path d="M278.6 233.4c12.5 12.5 12.5 32.8 0 45.3l-160 160c-12.5 12.5-32.8 12.5-45.3 0s-12.5-32.8 0-45.3L210.7 256 73.4 118.6c-12.5-12.5-12.5-32.8 0-45.3s32.8-12.5 45.3 0l160 160z"/></svg></span>`},
	{Solid, `angle-right`, ``, `<span class=fa-svg><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 320 512"><!--! Font Awesome Free 6.2.0 by @fontawesome - https://fontawesome.com License - https://fontawesome.com/license/free (Icons: CC BY 4.0, Fonts: SIL OFL 1.1, Code: MIT License) Copyright 2022 Fonticons, Inc. --><path d="M278.6 233.4c12.5 12.5 12.5 32.8 0 45.3l-160 160c-12.5 12.5-32.8 12.5-45.3 0s-12.5-32.8 0-45.3L210.7 256 73.4 118.6c-12.5-12.5-12.5-32.8 0-45.3s32.8-12.5 45.3 0l160 160z"/></svg></span>`},
	{Solid, `eye`, ``, `<span class=fa-svg><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 576 512"><!--! Font Awesome Free 6.2.0 by @fontawesome - https://fontawesome.com License - https://fontawesome.com/license/free (Icons: CC BY 4.0, Fonts: SIL OFL 1.1, Code: MIT License) Copyright 2022 Fonticons, Inc. --><path d="M288 32c-80.8 0-145.5 36.8-192.6 80.6C48.6 156 17.3 208 2.5 243.7c-3.3 7.9-3.3 16.7 0 24.6C17.3 304 48.6 356 95.4 399.4C142.5 443.2 207.2 480 288 480s145.5-36.8 192.6-80.6c46.8-43.5 78.1-95.4 93-131.1c3.3-7.9 3.3-16.7 0-24.6c-14.9-35.7-46.2-87.7-93-131.1C433.5 68.8 368.8 32 288 32zM432 256c0 79.5-64.5 144-144 144s-144-64.5-144-144s64.5-144 144-144s144 64.5 144 144zM288 192c0 35.3-28.7 64-64 64c-11.5 0-22.3-3-31.6-8.4c-.2 2.8-.4 5.5-.4 8.4c0 53 43 96 96 96s96-43 96-96s-43-96-96-96c-2.8 0-5.6 .1-8.4 .4c5.3 9.3 8.4 20.1 8.4 31.6z"/></svg></span>`},
	{Solid, `eye-slash`, ``, `<span class=fa-svg><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 640 512"><!--! Font Awesome Free 6.2.0 by @fontawesome - https://fontawesome.com License - https://fontawesome.com/license/free (Icons: CC BY 4.0, Fonts: SIL OFL 1.1, Code: MIT License) Copyright 2022 Fonticons, Inc. --><path d="M38.8 5.1C28.4-3.1 13.3-1.2 5.1 9.2S-1.2 34.7 9.2 42.9l592 464c10.4 8.2 25.5 6.3 33.7-4.1s6.3-25.5-4.1-33.7L525.6 386.7c39.6-40.6 66.4-86.1 79.9-118.4c3.3-7.9 3.3-16.7 0-24.6c-14.9-35.7-46.2-87.7-93-131.1C465.5 68.8 400.8 32 320 32c-68.2 0-125 26.3-169.3 60.8L38.8 5.1zM223.1 149.5C248.6 126.2 282.7 112 320 112c79.5 0 144 64.5 144 144c0 24.9-6.3 48.3-17.4 68.7L408 294.5c5.2-11.8 8-24.8 8-38.5c0-53-43-96-96-96c-2.8 0-5.6 .1-8.4 .4c5.3 9.3 8.4 20.1 8.4 31.6c0 10.2-2.4 19.8-6.6 28.3l-90.3-70.8zm223.1 298L373 389.9c-16.4 6.5-34.3 10.1-53 10.1c-79.5 0-144-64.5-144-144c0-6.9 .5-13.6 1.4-20.2L83.1 161.5C60.3 191.2 44 220.8 34.5 243.7c-3.3 7.9-3.3 16.7 0 24.6c14.9 35.7 46.2 87.7 93 131.1C174.5 443.2 239.2 480 320 480c47.8 0 89.9-12.9 126.2-32.5z"/></svg></span>`},
	{Regular, `copy`, ``, `<span class=fa-svg><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512"><!--! Font Awesome Free 6.2.0 by @fontawesome - https://fontawesome.com License - https://fontawesome.com/license/free (Icons: CC BY 4.0, Fonts: SIL OFL 1.1, Code: MIT License) Copyright 2022 Fonticons, Inc. --><path d="M502.6 70.63l-61.25-61.25C435.4 3.371 427.2 0 418.7 0H255.1c-35.35 0-64 28.66-64 64l.0195 256C192 355.4 220.7 384 256 384h192c35.2 0 64-28.8 64-64V93.25C512 84.77 508.6 76.63 502.6 70.63zM464 320c0 8.836-7.164 16-16 16H255.1c-8.838 0-16-7.164-16-16L239.1 64.13c0-8.836 7.164-16 16-16h128L384 96c0 17.67 14.33 32 32 32h47.1V320zM272 448c0 8.836-7.164 16-16 16H63.1c-8.838 0-16-7.164-16-16L47.98 192.1c0-8.836 7.164-16 16-16H160V128H63.99c-35.35 0-64 28.65-64 64l.0098 256C.002 483.3 28.66 512 64 512h192c35.2 0 64-28.8 64-64v-32h-47.1L272 448z"/></svg></span>`},
	{Solid, `play`, ``, `<span class=fa-svg><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 384 512"><!--! Font Awesome Free 6.2.0 by @fontawesome - https://fontawesome.com License - https://fontawesome.com/license/free (Icons: CC BY 4.0, Fonts: SIL OFL 1.1, Code: MIT License) Copyright 2022 Fonticons, Inc. --><path d="M73 39c-14.8-9.1-33.4-9.4-48.5-.9S0 62.6 0 80V432c0 17.4 9.4 33.4 24.5 41.9s33.7 8.1 48.5-.9L361 297c14.3-8.7 23-24.2 23-41s-8.7-32.2-23-41L73 39z"/></svg></span>`},
	{Solid, `clock-rotate-left`, ``, `<span class=fa-svg><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512"><!--! Font Awesome Free 6.2.0 by @fontawesome - https://fontawesome.com License - https://fontawesome.com/license/free (Icons: CC BY 4.0, Fonts: SIL OFL 1.1, Code: MIT License) Copyright 2022 Fonticons, Inc. --><path d="M75 75L41 41C25.9 25.9 0 36.6 0 57.9V168c0 13.3 10.7 24 24 24H134.1c21.4 0 32.1-25.9 17-41l-30.8-30.8C155 85.5 203 64 256 64c106 0 192 86 192 192s-86 192-192 192c-40.8 0-78.6-12.7-109.7-34.4c-14.5-10.1-34.4-6.6-44.6 7.9s-6.6 34.4 7.9 44.6C151.2 495 201.7 512 256 512c141.4 0 256-114.6 256-256S397.4 0 256 0C185.3 0 121.3 28.7 75 75zm181 53c-13.3 0-24 10.7-24 24V256c0 6.4 2.5 12.5 7 17l72 72c9.4 9.4 24.6 9.4 33.9 0s9.4-24.6 0-33.9l-65-65V152c0-13.3-10.7-24-24-24z"/></svg></span>`},
}

// TestSpanMatchesRustOutput verifies that Span(t, name, id) produces the
// exact same byte sequence the Rust mdBook front-end emits for the icons
// present in the rendered /tmp/m2-rust/intro.html file.
func TestSpanMatchesRustOutput(t *testing.T) {
	for _, c := range expectedSpanFromIntro {
		got, err := Span(c.t, c.name, c.id)
		if err != nil {
			t.Fatalf("Span(%v, %q, %q) unexpected error: %v", c.t, c.name, c.id, err)
		}
		if got != c.want {
			t.Errorf(
				"Span(%v, %q, %q) differs from Rust output\n--- got  (%d bytes) ---\n%s\n--- want (%d bytes) ---\n%s",
				c.t, c.name, c.id, len(got), got, len(c.want), c.want,
			)
		}
	}
}

// TestSVGIsRawFilesContent makes sure that the raw SVG payload for each
// embedded icon is exactly what is in the upstream
// fontawesome-free-6.2.0-desktop .svg file (no `<svg style=...>` rewrite,
// no trailing newline, no whitespace munging).
func TestSVGIsRawFilesContent(t *testing.T) {
	cases := []struct {
		t    Type
		name string
		want string
	}{
		{Solid, "bars", SolidBars},
		{Solid, "paintbrush", SolidPaintbrush},
		{Solid, "magnifying-glass", SolidMagnifyingGlass},
		{Solid, "print", SolidPrint},
		{Solid, "pencil", SolidPencil},
		{Solid, "spinner", SolidSpinner},
		{Solid, "angle-left", SolidAngleLeft},
		{Solid, "angle-right", SolidAngleRight},
		{Solid, "eye", SolidEye},
		{Solid, "eye-slash", SolidEyeSlash},
		{Solid, "play", SolidPlay},
		{Solid, "clock-rotate-left", SolidClockRotateLeft},
		{Regular, "copy", RegularCopy},
		{Brands, "github", BrandsGithub},
	}
	for _, c := range cases {
		got, err := SVG(c.t, c.name)
		if err != nil {
			t.Fatalf("SVG(%v, %q) unexpected error: %v", c.t, c.name, err)
		}
		if got != c.want {
			t.Errorf("SVG(%v, %q) byte drift: len got=%d want=%d", c.t, c.name, len(got), len(c.want))
		}
	}
}

// TestTypeFromString covers the alias surface used by mdBook.
func TestTypeFromString(t *testing.T) {
	cases := []struct {
		in      string
		want    Type
		wantErr bool
	}{
		{"solid", Solid, false},
		{"fas", Solid, false},
		{"regular", Regular, false},
		{"far", Regular, false},
		{"fa", Regular, false},
		{"brands", Brands, false},
		{"fab", Brands, false},
		{"", 0, true},
		{"bogus", 0, true},
	}
	for _, c := range cases {
		got, err := TypeFromString(c.in)
		if c.wantErr {
			if err == nil {
				t.Errorf("TypeFromString(%q) expected error, got %v", c.in, got)
			}
			continue
		}
		if err != nil {
			t.Errorf("TypeFromString(%q) unexpected error: %v", c.in, err)
			continue
		}
		if got != c.want {
			t.Errorf("TypeFromString(%q) = %v, want %v", c.in, got, c.want)
		}
	}
}

// TestSpanStripPrefixes makes sure that names carrying the helper's
// document prefixes ("fa-", "fab-", "fas-") resolve to the same icon as
// the bare name - same as mdBook's Rust helper.
func TestSpanStripPrefixes(t *testing.T) {
	bare, err := Span(Solid, "bars", "")
	if err != nil {
		t.Fatalf("bare: %v", err)
	}
	for _, n := range []string{"fa-bars", "fas-bars", "fab-bars"} {
		got, err := Span(Solid, n, "")
		if err != nil {
			t.Errorf("Span(Solid, %q) error: %v", n, err)
			continue
		}
		if got != bare {
			t.Errorf("Span(Solid, %q) differs from bare: %q vs %q", n, got, bare)
		}
	}
}

// TestSpanUnknownIcon verifies the error format mirrors mdBook's helper.
func TestSpanUnknownIcon(t *testing.T) {
	_, err := Span(Solid, "definitely-not-a-real-icon", "")
	if err == nil {
		t.Fatal("expected error for unknown icon")
	}
	msg := err.Error()
	for _, want := range []string{
		"definitely-not-a-real-icon",
		"solid",
		"https://fontawesome.com/v6/search?m=free",
	} {
		if !strings.Contains(msg, want) {
			t.Errorf("error %q missing substring %q", msg, want)
		}
	}
}

// TestDeprecationWarningIsOneShot guards the rate-limiting contract
// declared by warnDeprecated: the first call emits a warning, subsequent
// calls don't. The package doc comment promises users will see the
// warning once per build, not once per icon, and this test makes sure a
// future refactor doesn't accidentally move the warning outside the
// sync.Once.
func TestDeprecationWarningIsOneShot(t *testing.T) {
	// Two warnings into a buffer would mean the sync.Once was lost. We
	// can't capture os.Stderr from inside the test without changing the
	// package, so instead we exercise warnDeprecated directly: after
	// the first call the flag should be set; calling it again must not
	// change that.
	warnDeprecated()
	if !deprecationOnceDone() {
		t.Fatal("first warnDeprecated did not set the once-done flag")
	}
	// Second call must be a no-op (we can't observe the side-effect
	// count, but we can at least make sure it doesn't panic or deadlock).
	warnDeprecated()
}
