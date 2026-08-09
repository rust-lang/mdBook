# generate-github-markdown-css

> Generate the CSS for [`github-markdown-css`](https://github.com/sindresorhus/github-markdown-css)

## Get the CSS

See [`github-markdown-css`](https://github.com/sindresorhus/github-markdown-css)

## How

First the GitHub.com CSS is fetched. Then we walk through all rules that could take effect in Markdown content, and then do a custom cleanup. A [rendered Markdown](fixture.md) with all possible syntax is fetched from GitHub to check if we've done right.

## API

```js
import githubMarkdownCss from 'generate-github-markdown-css';

/*
If the `light` and `dark` themes are different the CSS returned will include `prefers-color-scheme` blocks for light and dark that match the specified `light` and `dark` themes (considered "auto" mode). This mode will always `preserveVars` as they are necessary for the `prefers-color-scheme` blocks

If the `light` and `dark` themes are equal the output will only contain one theme (considered "single" mode)

In "single" mode the output will apply the values of all variables to the rules themselves.The output will not contain any `var(--variable)` statements. You can disable this by setting `preserveVariables` to true
*/

console.log(await githubMarkdownCss({
		// The theme to use for light theme.
		light: 'light',
		// The theme to use for dark theme.
		dark: 'dark',
		// If `true`, will return a list of available themes instead of the CSS.
		list: false,
		// If `true`, will preserve the block of variables for a given theme even if
		// only exporting one theme. By default, variables are applied to the rules
		// themselves and the resulting CSS will not contain any `var(--variable)`.
		preserveVariables: false,
		// Only output the color variables part of the CSS. Forces
		// `preserveVariables` to be `true`.
		onlyVariables: false,
		// Only output the style part of the CSS without any variables. Forces
		// `preserveVariables` to be `true` and ignores the theme values.
		// Useful to get the base styles to use multiple themes.
		onlyStyles: false,
		// Include extra styles from GitHub Flavored Markdown, like code snippets.
		useFixture: true,
		// Set the root selector of the rendered Markdown body as it should appear
		// in the output CSS. Defaults to `.markdown-body`.
		rootSelector: '.markdown-body',
		// Make the background transparent instead of white/black. Useful when
		// embedding the Markdown content in a page with a custom background.
		transparentBackground: false,
	}
));
//=> '.markdown-body { …'
```

## CLI

```sh
npm install --global generate-github-markdown-css
```

```
$ github-markdown-css --help

  Usage
    github-markdown-css > <filename>

  Options
    --list                     List available themes

    Set theme:
    --light                    Light theme name from --list values
    --dark                     Dark theme name from --list values
    --theme                    Theme name: 'auto', light', 'dark', or another from --list values.
                               'auto' means using the media query (prefers-color-scheme)
                               to switch between the 'light' and 'dark' theme.

    Output options:
    --preserve-variables       Preserve variables in the output. Only applies if light
                               and dark themes match or if type is not 'auto'
    --only-style               Only output the styles, forces --preserve-variables on
    --only-variables           Only output the variables for the specified themes
    --no-use-fixture           Exclude generated classes that come from GitHub Markdown API rendered fixture.md
    --root-selector            Specify the root selector when outputting styles, default '.markdown-body'
    --transparent-background   Make the background transparent instead of white/black

  Examples
    $ github-markdown-css --list
    light
    dark
    dark_dimmed
    dark_high_contrast
    dark_colorblind
    light_colorblind

    $ github-markdown-css --light=light --dark=dark
    [CSS with variable blocks for 'light' and 'dark' themes]

    $ github-markdown-css --theme=dark_dimmed --only-variables
    [CSS with single variable block for 'dark_dimmed' theme with no element styles]

    $ github-markdown-css --only-style
    [CSS with only element styles using variables but no variables set.
      Use in combination with output from setting --only-variables]
```
