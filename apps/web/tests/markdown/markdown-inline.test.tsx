/* @layer renderer-app @kind test */
import { cleanup, fireEvent, render } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { BoardMarkdown } from '../../src/shell';
import { location, renderMarkdown, texts } from './render-markdown';

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

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
  it('the text may carry markup', () => {
    const root = renderMarkdown('[**docs**](https://example.com/a?x=1&y=2 "The docs") and [m](mailto:a@b.co)');
    const [docs, mail] = [...root.querySelectorAll('a')];
    expect(docs?.getAttribute('href')).toBe('https://example.com/a?x=1&y=2');
    expect(docs?.querySelector('strong')?.textContent).toBe('docs');
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

  it('a link to a proof under /files/ stays a link and opens in a new tab', () => {
    const open = vi.spyOn(window, 'open').mockReturnValue(null);
    const link = renderMarkdown('[f](/files/p/proofs/a.png)').querySelector('a');
    expect(link?.getAttribute('href')).toBe('/files/p/proofs/a.png');
    fireEvent.click(link as Element);
    expect(open).toHaveBeenCalledWith('/files/p/proofs/a.png', '_blank', 'noopener,noreferrer');
  });

  it('an https link opens in a new tab and a path with dots up never leaves the files folder', () => {
    const open = vi.spyOn(window, 'open').mockReturnValue(null);
    fireEvent.click(renderMarkdown('[w](https://example.com/a)').querySelector('a') as Element);
    expect(open).toHaveBeenCalledWith('https://example.com/a', '_blank', 'noopener,noreferrer');
    open.mockClear();
    fireEvent.click(renderMarkdown('[w](/files/../etc/x)').querySelector('a') as Element);
    expect(open).not.toHaveBeenCalled();
  });

  it('an app path to a board goes through the router and does not reload the page', () => {
    const open = vi.spyOn(window, 'open').mockReturnValue(null);
    const root = renderMarkdown('[other](/p/other#kanban)');
    fireEvent.click(root.querySelector('a') as Element);
    expect(open).not.toHaveBeenCalled();
    expect(location()).toBe('/p/other#kanban');
  });
});

describe('markdown images and ids', () => {
  it('outside a router the images still draw and the ids stay text', () => {
    const { container } = render(<BoardMarkdown>{'![a](/files/p/proofs/a.png) and T-5'}</BoardMarkdown>);
    expect(container.querySelector('img')?.getAttribute('src')).toBe('/files/p/proofs/a.png');
    expect(container.querySelector('a')).toBeNull();
  });

  it('an image from /files or https is drawn', () => {
    const root = renderMarkdown('![shot](/files/p/proofs/p-1.png) ![s](https://x.test/a.png "t")');
    expect([...root.querySelectorAll('img')].map((image) => image.getAttribute('src'))).toEqual(['/files/p/proofs/p-1.png', 'https://x.test/a.png']);
  });

  it('an image under /files with dots up drops to its alt text', () => {
    const root = renderMarkdown('![x](/files/../secret.png)');
    expect(root.querySelector('img')).toBeNull();
    expect(root.textContent).toBe('x');
  });

  it.fails('gap: a data image is drawn (Tessera draws only http, https and app paths)', () => {
    expect(renderMarkdown('![x](data:image/png;base64,AAAA)').querySelector('img')).not.toBeNull();
  });

  it.fails('gap: a drawn image opens in the lightbox (Tessera Markdown images are not clickable)', () => {
    const root = renderMarkdown('![shot](/files/p/proofs/p-1.png)');
    fireEvent.click(root.querySelector('img') as Element);
    expect(document.body.querySelector('[role="dialog"]')).not.toBeNull();
  });

  it('an image from any other address drops to its alt text', () => {
    for (const bad of ['![x](http://x.test/a.png)', '![x](javascript:alert(1))', '![x](data:image/svg+xml;base64,PHN2Zz4=)', '![x](//evil.test/a.png)', '![x](file:///c:/a.png)']) {
      const root = renderMarkdown(bad);
      expect(root.querySelector('img')).toBeNull();
      expect(root.textContent).toBe('x');
    }
  });

  it('ids T-5, I-3, A-2 and H-1 link to their board anchors and the look-alikes stay text', () => {
    const root = renderMarkdown('Fixed T-5 (see I-3, A-2 and H-1) but not XT-5, T-5x, T-5-9, #T-5 or t-5.');
    expect([...root.querySelectorAll('a')].map((link) => link.getAttribute('href'))).toEqual(['/p/demo#T-5', '/p/demo#I-3', '/p/demo#A-2', '/p/demo#H-1']);
  });

  it('an id link moves the board to its anchor without a reload and stays text in code and links', () => {
    const open = vi.spyOn(window, 'open').mockReturnValue(null);
    const root = renderMarkdown('see I-3 and `T-5` and [A-2](https://a.test)');
    expect(root.querySelectorAll('a')).toHaveLength(2);
    fireEvent.click(root.querySelector('a') as Element);
    expect(location()).toBe('/p/demo#I-3');
    expect(open).not.toHaveBeenCalled();
  });
});
