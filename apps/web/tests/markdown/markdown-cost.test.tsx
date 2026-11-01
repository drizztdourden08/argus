/* @layer renderer-app @kind test */
import { cleanup } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import { renderMarkdown } from './render-markdown';

afterEach(cleanup);

const N = 12000;

const HOSTILE: Record<string, string> = {
  stars: '*a '.repeat(N / 3),
  underscores: '_a_ '.repeat(N / 4),
  mixed_open: '*_~~'.repeat(N / 4),
  mixed_close: '*a _a '.repeat(N / 6),
  open_brackets: '['.repeat(N),
  broken_links: '[a](b'.repeat(N / 5),
  nested_links: '[[[[a](b)](c)](d)]'.repeat(N / 18),
  images: '![a](b'.repeat(N / 6),
  backticks: '`'.repeat(N),
  backtick_runs: '` `` ``` ````'.repeat(N / 14),
  long_url: `https://${'a'.repeat(N)}`,
  urls: 'https://a.test/x '.repeat(N / 17),
  lists: '- '.repeat(N / 40),
  deep_lists: Array.from({ length: 100 }, (_, index) => `${' '.repeat(index * 2)}- x`).join('\n'),
  list_blank: '- a\n\n'.repeat(N / 6),
  spaces: ' '.repeat(N),
  newlines: '\n'.repeat(N),
  table: `| a | b |\n|---|---|\n${'| 1 | 2 |\n'.repeat(N / 10)}`,
  pipes: '|'.repeat(N),
  fences: '```\n'.repeat(N / 4),
  headings: '# a\n'.repeat(N / 4),
  backslashes: '\\'.repeat(N),
  entities: '&<>"\''.repeat(N / 5),
  unicode: '\u{1F600}*'.repeat(N / 3),
};

describe('K4 markdown on hostile input', { tags: ['standard'] }, () => {
  it.each(Object.entries(HOSTILE))('finishes within the test limit and leaves no script or event attribute: %s', (name, source) => {
    const root = renderMarkdown(source);
    expect(root.querySelector('script, [onclick], [onerror]'), name).toBeNull();
  });
});
