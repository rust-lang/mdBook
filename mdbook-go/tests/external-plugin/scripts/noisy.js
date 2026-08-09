#!/usr/bin/env node
// noisy.js — mdbook-go external preprocessor fixture (supports-probe edge case).
//
// This preprocessor advertises support for every renderer *except* a small
// blacklist. Both Rust and Go must agree on the exit code for the probe.
'use strict';

const BLACKLIST = new Set(['not-supported']);

function main() {
  const argv = process.argv.slice(2);
  if (argv[0] === 'supports') {
    const renderer = argv[1] || '';
    process.exit(BLACKLIST.has(renderer) ? 1 : 0);
  }

  // No-op transformation: pass the book through unchanged.
  let raw = '';
  process.stdin.setEncoding('utf8');
  process.stdin.on('data', (chunk) => { raw += chunk; });
  process.stdin.on('end', () => {
    const input = JSON.parse(raw);
    process.stdout.write(JSON.stringify(input[1]) + '\n');
  });
}

main();