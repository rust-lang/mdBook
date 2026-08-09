#!/usr/bin/env node
// banner.js — mdbook-go external preprocessor fixture.
//
// Protocol:
//   supports <renderer>   argv[2] is the renderer name; exit 0 if supported.
//   otherwise             read JSON tuple [ctx, book] from stdin, modify the
//                         book, write the modified book JSON to stdout.
//
// Behaviour: prepend a single blockquote banner line to every chapter's
// `content` field. Draft chapters (content === "" and no source file) are
// left alone.
'use strict';

const BANNER = '> Banner was here\n\n';

function isChapter(item) {
  return item && typeof item === 'object' && 'Chapter' in item;
}

function walk(items, fn) {
  for (const it of items) {
    if (isChapter(it)) fn(it.Chapter);
    if (it && it.Chapter && Array.isArray(it.Chapter.sub_items)) {
      walk(it.Chapter.sub_items, fn);
    }
  }
}

function main() {
  const argv = process.argv.slice(2);
  if (argv[0] === 'supports') {
    const renderer = argv[1] || '';
    process.exit(renderer === 'html' ? 0 : 1);
  }

  let raw = '';
  process.stdin.setEncoding('utf8');
  process.stdin.on('data', (chunk) => { raw += chunk; });
  process.stdin.on('end', () => {
    const input = JSON.parse(raw);
    // input is the 2-element tuple [ctx, book].
    const book = input[1];
    walk(book.items, (ch) => {
      if (typeof ch.content === 'string' && ch.content.length > 0) {
        ch.content = BANNER + ch.content;
      }
    });
    process.stdout.write(JSON.stringify(book) + '\n');
  });
}

main();