# Testing relative links for the print page

When we link to [the first section](../first/nested.md), it should work on
both the print page and the non-print page.

The same link should work [with an html extension](../first/nested.html).

A [fragment link](#some-section) should work.

Link [outside](../../std/foo/bar.html).

Link [outside with anchor](../../std/foo/bar.html#panic).

Link [inside but doesn't exist](../first/alpha/beta.md).
Link [inside but doesn't exist with anchor](../first/alpha/beta.md#anchor).
Link [inside to html](../first/alpha/gamma.html).
Link [inside to html with anchor](../first/alpha/gamma.html#anchor).

![Some image](../images/picture.png)

<a href="../first/nested.md">HTML Link</a>

<img src="../images/picture.png" alt="raw html">

## Some section

[Links with scheme shouldn't be touched.](https://example.com/foo.html#bar)

<a href="../images/not-html?arg1&arg2#with-anchor">Non-html link</a>
