# Inline HTML

## Comments

<!--comment-->

<!--

**bigger**

comment

-->

## Void elements

<map name="my-map">
  <area shape="rect" coords="0,0,10,20" href="https://example.com/" alt="alt text">
</map>

Line<br>break

Word<wbr>break

<table>
  <colgroup>
    <col>
    <col span="2" class="a">
  </colgroup>
</table>

<embed
  type="image/jpeg"
  src="/image.jpg"
  width="100"
  height="200">

Rule:
<hr>

<img src="example.jpg">

<input type="text">

<link href="example.css" rel="stylesheet">

<meta name="example"
  content="Example content">

<video>
  <source src="video.webm" type="video/webm">
  <track kind="captions" src="captions.vtt" srclang="en">
</video>

## Blocks

<div>
A block HTML element trying to do *markup*.
</div>

<div>

A block HTML with spaces that **cause** it to be interleaved with markdown.

</div>

## Scripts

<script></script>

<script async src="foo.js"></script>

<script>
const x = 'some *text* inside';

*don't* < try to "parse me
</script>

## Style

<style>

.foo {
    background-color: red;
}

/* **don't** < try to "parse me
*/

</style>

<style media="(width < 500px)">
.bar { background-color: green }
</style>
