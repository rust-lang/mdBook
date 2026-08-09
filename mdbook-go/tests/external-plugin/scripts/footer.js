#!/usr/bin/env node
// footer.js — mdbook-go external preprocessor fixture (second in chain).
//
// Mirrors banner.js but appends a footer line. Together they exercise the
// `before`/`after` topological ordering rules in the registry.
'use strict';

const FOOTER = '\n\n> Footer was here\n';

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
    // Honour the `renderers = ["html"]` whitelist by returning success only
    // for the html backend.
    process.exit(renderer === 'html' ? 0 : 1);
  }

  let raw = '';
  process.stdin.setEncoding('utf8');
  process.stdin.on('data', (chunk) => { raw += chunk; });
  process.stdin.on('end', () => {
    const input = JSON.parse(raw);
    const book = input[1];
    walk(book.items, (ch) => {
      if (typeof ch.content === 'string' && ch.content.length > 0) {
        ch.content = ch.content + FOOTER;
      }
    });
    process.stdout.write(JSON.stringify(book) + '\n');
  });
}

main();