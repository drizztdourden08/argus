/* @layer renderer-app @kind test */
import { cleanup } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import { renderMarkdown, texts } from './render-markdown';

afterEach(cleanup);

describe('markdown inline marks', () => {
  it('bold, italic, strike and code, in both mark styles', () => {
    const root = renderMarkdown('**b** __b__ *i* _i_ ~~s~~ `c`');
    expect(texts(root, 'strong')).toEqual(['b', 'b']);
    expect(texts(root, 'em')).toEqual(['i', 'i']);
    expect(texts(root, 's')).toEqual(['s']);
    expect(texts(root, 'code')).toEqual(['c']);
  });

  it('nested and combined emphasis', () => {
    expect(renderMarkdown('**a *b* c**').querySelector('strong em')?.textContent).toBe('b');
    expect(renderMarkdown('*a **b** c*').querySelector('em strong')?.textContent).toBe('b');
    expect(renderMarkdown('***both***').querySelectorAll('em, strong')).toHaveLength(2);
    expect(renderMarkdown('**bold and ~~struck~~**').querySelector('strong s')?.textContent).toBe('struck');
  });

  it('intraword underscores stay, unmatched marks stay, spaces inside block emphasis', () => {
    const intraword = renderMarkdown('snake_case_name and 2*3*4');
    expect(texts(intraword, 'em')).toEqual(['3']);
    expect(intraword.textContent).toBe('snake_case_name and 234');
    expect(renderMarkdown('**never closed').textContent).toBe('**never closed');
    expect(renderMarkdown('a * b * c').textContent).toBe('a * b * c');
    expect(texts(renderMarkdown('~one~ ~~two~~'), 's')).toEqual(['one', 'two']);
  });

  it('code spans take everything literally, including marks and html', () => {
    const root = renderMarkdown('`<b>**x**</b>` and ``a ` b``');
    expect(texts(root, 'code')).toEqual(['<b>**x**</b>', 'a ` b']);
    expect(root.querySelector('b, strong')).toBeNull();
  });

  it('backslash escapes', () => {
    expect(renderMarkdown(String.raw`\*not italic\* \# no \[link\](x) \\ end`).textContent).toBe('*not italic* # no [link](x) \\ end');
    expect(renderMarkdown(String.raw`\<b\> and \>`).textContent).toBe('<b> and >');
  });
});

describe('markdown links', () => {
  it('the text may carry markup and an external link opens in a new tab safely', () => {
    const root = renderMarkdown('[**docs**](https://example.com/a?x=1&y=2 "The docs") and [m](mailto:a@b.co)');
    const [docs, mail] = [...root.querySelectorAll('a')];
    expect(docs?.getAttribute('href')).toBe('https://example.com/a?x=1&y=2');
    expect(docs?.querySelector('strong')?.textContent).toBe('docs');
    expect([docs?.getAttribute('target'), docs?.getAttribute('rel')]).toEqual(['_blank', 'noopener noreferrer']);
    expect(mail?.getAttribute('href')).toBe('mailto:a@b.co');
  });

  it('a link with parentheses in its address', () => {
    const link = renderMarkdown('[w](https://en.wikipedia.org/wiki/Foo_(bar))').querySelector('a');
    expect(link?.getAttribute('href')).toBe('https://en.wikipedia.org/wiki/Foo_(bar)');
    expect(link?.textContent).toBe('w');
  });

  it('bare addresses become links without the closing punctuation, but not inside code', () => {
    const root = renderMarkdown('See https://example.com/a_b_c. And (https://example.com/x) or https://example.com/p?q=1&r=2, `https://not.linked`');
    expect([...root.querySelectorAll('a')].map((link) => link.getAttribute('href'))).toEqual([
      'https://example.com/a_b_c',
      'https://example.com/x',
      'https://example.com/p?q=1&r=2',
    ]);
    expect(root.querySelector('p')?.textContent).toContain('/a_b_c. And (https://example.com/x) or');
    expect(texts(root, 'code')).toEqual(['https://not.linked']);
  });

  it('a bare address inside link text makes no second link, and other schemes are not linked', () => {
    expect(renderMarkdown('[https://a.test/x](https://b.test/y)').querySelectorAll('a')).toHaveLength(1);
    expect(renderMarkdown('xhttps://a.test').querySelector('a')).toBeNull();
    expect(renderMarkdown('javascript:alert(1) and ftp://x.test').querySelector('a')).toBeNull();
  });

  it.fails('gap: a link to a proof under /files/ stays a link', () => {
    expect(renderMarkdown('[f](/files/p/proofs/a.png)').querySelector('a')?.getAttribute('href')).toBe('/files/p/proofs/a.png');
  });
});

describe('markdown images and ids', () => {
  it.fails('gap: an image from /files, https or a data image is a clickable img', () => {
    const root = renderMarkdown('![shot](/files/p/proofs/p-1.png) ![s](https://x.test/a.png "t")');
    expect(root.querySelectorAll('img')).toHaveLength(2);
  });

  it('an image from any other address drops to its alt text', () => {
    for (const bad of ['![x](http://x.test/a.png)', '![x](javascript:alert(1))', '![x](data:image/svg+xml;base64,PHN2Zz4=)', '![x](//evil.test/a.png)', '![x](file:///c:/a.png)']) {
      const root = renderMarkdown(bad);
      expect(root.querySelector('img')).toBeNull();
      expect(root.textContent).toBe('x');
    }
  });

  it.fails('gap: ids T-5, I-3, A-2 and H-1 link to page anchors and the look-alikes stay text', () => {
    const root = renderMarkdown('Fixed T-5 (see I-3, A-2 and H-1) but not XT-5, T-5x, T-5-9, #T-5 or t-5.');
    expect([...root.querySelectorAll('a')].map((link) => link.getAttribute('href'))).toEqual(['#T-5', '#I-3', '#A-2', '#H-1']);
  });
});
