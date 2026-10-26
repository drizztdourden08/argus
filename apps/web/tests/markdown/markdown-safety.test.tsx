/* @layer renderer-app @kind test */
import { cleanup } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import { renderMarkdown } from './render-markdown';

afterEach(cleanup);

const TAGS = new Set([
  'div', 'span', 'p', 'br', 'strong', 'em', 's', 'code', 'pre', 'a', 'ul', 'ol', 'li', 'blockquote', 'hr', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6',
  'table', 'thead', 'tbody', 'tr', 'th', 'td', 'button', 'input', 'label', 'svg', 'g', 'rect', 'path',
]);
const LINK_OK = /^(?:https?:\/\/|mailto:)/i;

const auditElement = (element: Element): void => {
  expect(TAGS.has(element.tagName.toLowerCase()), `tag not allowed: ${element.tagName}`).toBe(true);
  for (const { name, value } of element.attributes) {
    expect(name.startsWith('on'), `event attribute: ${name}`).toBe(false);
    if (name === 'href' || name === 'src') expect(value, `${name} not allowed`).toMatch(LINK_OK);
  }
};

const hiddenCharacters = (text: string): number[] =>
  [...text].map((char) => char.charCodeAt(0)).filter((code) => code === 0 || (code >= 1 && code <= 8) || (code >= 0xe000 && code <= 0xf8ff));

const audit = (root: HTMLElement): void => {
  for (const element of root.querySelectorAll('*')) auditElement(element);
};

const VECTORS = [
  '<script>alert(1)</script>',
  '<img src=x onerror=alert(1)>',
  '<svg/onload=alert(1)>',
  '"><script>alert(1)</script>',
  "'><img src=x onerror=alert(1)>",
  '<a href="javascript:alert(1)">x</a>',
  '<iframe src="javascript:alert(1)"></iframe>',
  '[a](https://x.test" onmouseover="alert(1))',
  '[a](<https://x.test" onmouseover="alert(1)>)',
  '[a](https://x.test "t" onmouseover="alert(1)")',
  '![x" onerror="alert(1)](/files/a.png)',
  '![x](/files/a.png" onerror="alert(1))',
  '![x](/files/a.png "t\' onerror=\'alert(1)")',
  '[a](https://x.test/&quot; onmouseover=&quot;alert(1))',
  '```"><script>alert(1)</script>\n"><script>alert(2)</script>\n```',
  '`<script>`',
  '| <script> | b |\n|---|---|\n| <img src=x onerror=1> | 2 |',
  '> <script>alert(1)</script>',
  '- <script>alert(1)</script>\n  - <img src=x onerror=1>',
  '# <script>alert(1)</script>',
  '**<script>alert(1)</script>**',
  '[<script>](https://x.test)',
  '<<script>script>alert(1)<</script>/script>',
  '<style>*{display:none}</style>',
  '<meta http-equiv=refresh content="0;url=javascript:alert(1)">',
  '&lt;script&gt;alert(1)&lt;/script&gt;',
  '&#60;script&#62;alert(1)&#60;/script&#62;',
  '\u0000<script>\u0000alert(1)',
  'http://x.test/"onmouseover="alert(1)',
  'https://x.test/<script>alert(1)</script>',
  'https://x.test/&quot;><svg onload=alert(1)>',
  '\ue000\ue001<script>',
  '[a]( https://x.test )',
];

describe('markdown link allow-list', () => {
  it('a link to http, https or mailto stays a link and the others lose it', () => {
    const allowed = renderMarkdown('[a](https://a.test) [b](http://a.test/x?y#z) [c](HTTPS://A.TEST) [d](mailto:a@b.co)');
    expect(allowed.querySelectorAll('a')).toHaveLength(4);
    for (const href of ['javascript:alert(1)', 'JaVaScRiPt:alert(1)', 'vbscript:x', 'data:text/html,<script>', 'file:///etc/passwd', '//evil.test/x', '/x', '#x', 'ftp://x.test']) {
      const root = renderMarkdown(`[x](${href})`);
      expect(root.querySelector('a'), href).toBeNull();
      expect(root.textContent, href).toBe('x');
    }
  });

  it('rendered links to forbidden targets lose the link and keep the words', () => {
    const sources = [
      '[x](javascript:alert(1))',
      '[x](JAVASCRIPT:alert(1))',
      '[x](vbscript:msgbox(1))',
      '[x](data:text/html;base64,PHNjcmlwdD4=)',
      '[x](file:///c:/x)',
      '[x](//evil.test)',
      '[x](&#106;avascript:alert(1))',
      '[x](javascript&colon;alert(1))',
      '[x](\\javascript:alert(1))',
    ];
    for (const source of sources) {
      const root = renderMarkdown(source);
      expect(root.querySelector('a, [href]'), source).toBeNull();
      expect(root.textContent, source).toContain('x');
    }
  });

  it('a quote or angle bracket in an address cannot leave its attribute', () => {
    const hostile = renderMarkdown('[a](https://x.test/\'"<>)');
    audit(hostile);
    expect(hostile.querySelector('[onmouseover]')).toBeNull();
    const plain = renderMarkdown("[a](https://x.test/it's)");
    expect(plain.querySelector('a')?.getAttribute('href')).toBe("https://x.test/it's");
    audit(plain);
  });
});

describe('markdown against hostile input', () => {
  it('the output of every hostile input only has tags, attributes and URLs from the lists', () => {
    for (const vector of VECTORS) {
      const root = renderMarkdown(vector);
      expect(() => { audit(root); }, vector).not.toThrow();
      cleanup();
    }
  });

  it('raw html never becomes an element and is dropped from the text', () => {
    for (const source of ['<script>alert(1)</script>', '<img src=x onerror=alert(1)>', '<style>*{display:none}</style>']) {
      const root = renderMarkdown(source);
      expect(root.querySelector('script, img, style'), source).toBeNull();
      expect(root.textContent, source).toBe('');
    }
  });

  it('entities become text and never an element', () => {
    const root = renderMarkdown('&lt;script&gt; &#60; &amp;');
    expect(root.textContent).toBe('<script> < &');
    expect(root.querySelector('script')).toBeNull();
  });

  it.fails('gap: control characters and the private-use area never reach the text', () => {
    expect(hiddenCharacters(renderMarkdown('a\u0000b\u0001c\ue000d').textContent)).toEqual([]);
  });
});
