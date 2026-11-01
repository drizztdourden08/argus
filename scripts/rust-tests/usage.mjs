/* @layer tooling-scripts @kind logic */
const usage = [
  'usage: node scripts/rust-tests.mjs --tag critical|standard|e2e (--changed [--base <ref>] | --files <file>...) [--depth <n>] [--dry-run]',
  '       node scripts/rust-tests.mjs --check-tags',
].join('\n');

export { usage };
