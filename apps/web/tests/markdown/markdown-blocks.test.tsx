/* @layer renderer-app @kind test */
import { cleanup } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import { codeLines, renderMarkdown, texts } from './render-markdown';

afterEach(cleanup);

describe('S1 markdown blocks', { tags: ['standard'] }, () => {
  it('empty and blank input render to nothing', () => {
    for (const source of ['', '  \n \n']) expect(renderMarkdown(source).textContent).toBe('');
  });

  it('headings one to five follow the offset; a hashtag and a closing run of hashes are not headings', () => {
    const root = renderMarkdown('# One\n## Two\n### Three\n#### Four\n##### Five\n#tag');
    expect(texts(root, 'h2')).toEqual(['One']);
    expect(texts(root, 'h3')).toEqual(['Two']);
    expect(texts(root, 'h4')).toEqual(['Three']);
    expect(texts(root, 'h5')).toEqual(['Four']);
    expect(texts(root, 'h6')).toEqual(['Five']);
    expect(texts(root, 'p')).toEqual(['#tag']);
    expect(texts(renderMarkdown('## Title ##'), 'h3')).toEqual(['Title']);
  });

  it('paragraphs split on blank lines and a soft break stays a line break in the text', () => {
    expect(texts(renderMarkdown('one\ntwo\n\nthree'), 'p')).toEqual(['one\ntwo', 'three']);
  });

  it('every form of a horizontal rule is a separator', () => {
    for (const source of ['---', '***', '___', '- - -', '  ***  ']) {
      expect(renderMarkdown(`a\n\n${source}\n\nb`).querySelectorAll('[role="separator"]')).toHaveLength(1);
    }
  });

  it('blockquotes carry markdown and nest, and a lazy line continues the quote paragraph', () => {
    const root = renderMarkdown('> quoted **text**\n> second\n>\n> > deeper');
    const outer = root.querySelector('blockquote');
    expect(outer?.querySelector('strong')?.textContent).toBe('text');
    expect(outer?.querySelector('blockquote')?.textContent.trim()).toBe('deeper');
    expect(texts(renderMarkdown('> one\ntwo'), 'blockquote p')).toEqual(['one\ntwo']);
  });

  it('fenced code keeps its body as text and carries a Copy code button', () => {
    const root = renderMarkdown('```js\nconst a = "<b>" && 1;\n```');
    expect(root.querySelector('pre code')?.textContent).toBe('const a = "<b>" && 1;');
    expect(root.querySelector('button[aria-label="Copy code"]')).not.toBeNull();
    expect(root.querySelector('script, b')).toBeNull();
  });

  it('a fence keeps markdown inside as text, ~~~ fences work and a hostile info string stays inert', () => {
    const root = renderMarkdown('~~~ py"><script>\n**not bold**\n~~~');
    expect(root.querySelector('pre code')?.textContent).toBe('**not bold**');
    expect(root.querySelector('strong, script')).toBeNull();
  });

  it('an unclosed fence runs to the end of the text', () => {
    const root = renderMarkdown('before\n\n```\nline 1\nline 2');
    expect(codeLines(root)).toEqual(['line 1', 'line 2']);
    expect(texts(root, 'p')).toEqual(['before']);
  });

  it('a fence needs a closing run at least as long', () => {
    const root = renderMarkdown('````\n```\nstill code\n````\nafter');
    expect(codeLines(root)).toEqual(['```', 'still code']);
    expect(texts(root, 'p')).toEqual(['after']);
  });
});

describe('S1 markdown lists', { tags: ['standard'] }, () => {
  it('bullet and numbered lists, and a numbered list keeps its first number', () => {
    expect(texts(renderMarkdown('- a\n- b\n* c'), 'ul > li')).toEqual(['a', 'b', 'c']);
    expect(texts(renderMarkdown('1. a\n2. b'), 'ol > li')).toEqual(['a', 'b']);
    expect(renderMarkdown('3. a\n4. b').querySelector('ol')?.getAttribute('start')).toBe('3');
  });

  it('nested lists by indentation, mixed kinds', () => {
    const root = renderMarkdown('- a\n  - b\n    - c\n  - d\n- e\n\n1. x\n   - y\n2. z');
    expect(root.querySelectorAll('ul ul ul')).toHaveLength(1);
    const second = [...(root.querySelector('ul ul')?.children ?? [])];
    expect(second.map((item) => (item.firstChild?.textContent ?? '').trim())).toEqual(['b', 'd']);
    expect(root.querySelector('ol > li ul > li')?.textContent).toBe('y');
  });

  it('a loose list wraps its items in paragraphs', () => {
    expect(texts(renderMarkdown('- a\n\n- b'), 'li p')).toEqual(['a', 'b']);
  });

  it('a list can interrupt a paragraph; "2." in prose does not', () => {
    const steps = renderMarkdown('Steps:\n- one\n- two');
    expect(texts(steps, 'p')).toEqual(['Steps:']);
    expect(steps.querySelectorAll('li')).toHaveLength(2);
    const prose = renderMarkdown('Version\n2. beta');
    expect(prose.querySelector('ol')).toBeNull();
    expect(texts(prose, 'p')).toEqual(['Version\n2. beta']);
  });

  it('task lists render disabled checkboxes, checked for x and X, and a plain item has none', () => {
    const root = renderMarkdown('- [ ] open\n- [x] done\n- [X] also done\n- plain');
    const boxes = [...root.querySelectorAll<HTMLInputElement>('input[type="checkbox"]')];
    expect(boxes.map((box) => box.checked)).toEqual([false, true, true]);
    expect(boxes.every((box) => box.disabled)).toBe(true);
    expect(root.querySelectorAll('li')).toHaveLength(4);
    expect(root.querySelectorAll('li')[3]?.querySelector('input')).toBeNull();
  });

  it('a code block inside a list item stays in the item', () => {
    const root = renderMarkdown('- item\n\n  ```\n  code\n  ```\n- next');
    expect(root.querySelector('li pre code')?.textContent).toBe('code');
    expect(root.querySelectorAll('li')).toHaveLength(2);
  });
});

describe('S1 markdown tables', { tags: ['standard'] }, () => {
  const table = '| Name | Qty | Note |\n|:-----|:---:|-----:|\n| **a** | 1 | x \\| y |\n| b | 2 |\n| c | 3 | z | extra |';

  it('keep inline markup in cells, an escaped pipe, ragged rows and cut-off extra cells', () => {
    const root = renderMarkdown(table);
    const head = [...root.querySelectorAll('thead th')];
    expect(head.map((cell) => cell.textContent)).toEqual(['Name', 'Qty', 'Note']);
    const rows = [...root.querySelectorAll('tbody tr')].map((row) => texts(row, 'td'));
    expect(rows).toEqual([['a', '1', 'x | y'], ['b', '2', ''], ['c', '3', 'z']]);
    expect(root.querySelector('tbody strong')?.textContent).toBe('a');
    expect(root.textContent).not.toContain('extra');
  });

  it('a header without its delimiter row is a paragraph', () => {
    const root = renderMarkdown('| a | b |\nplain');
    expect(root.querySelector('table')).toBeNull();
    expect(texts(root, 'p')).toEqual(['| a | b |\nplain']);
  });
});
