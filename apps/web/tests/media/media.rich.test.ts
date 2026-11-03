/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { checkField, richTextCount } from '../../src/data/content';

const PARAGRAPH = 'This is a plain paragraph that runs on well past the short idea limit of a hundred and twenty characters so it must be refused.';

describe('P8 rich description count and refusal', { tags: ['critical'] }, () => {
  it('counts the text only: image lines and bullet markers are left out', () => {
    expect(richTextCount('- one\n- two')).toBe(6);
    expect(richTextCount('lead\n![the form](img:1)\n- point\n\n![other](img:2)')).toBe(9);
    expect(richTextCount('')).toBe(0);
  });

  it('refuses a lone paragraph with its key and the way out, and accepts it as bullets or with a screenshot', () => {
    const refused = checkField('description', PARAGRAPH);
    expect(refused).toMatchObject({ ok: false, key: 'rich-lone-paragraph' });
    expect(refused.ok ? '' : refused.message).toContain('bullets');
    expect(checkField('description', `- ${PARAGRAPH.slice(0, 100)}`).ok).toBe(true);
    expect(checkField('description', 'A short idea').ok).toBe(true);
    expect(checkField('description', `${PARAGRAPH.slice(0, 60)}\n![the form](img:1)`, { images: [1] }).ok).toBe(true);
  });

  it('refuses text over 500 characters but lets images through, and refuses an image the item does not have', () => {
    const bullets = Array.from({ length: 4 }, () => `- ${'x'.repeat(150)}`).join('\n');
    expect(checkField('description', bullets)).toMatchObject({ ok: false, key: 'rich-text-long' });
    expect(checkField('description', `- ok\n![a](img:1)\n![b](img:2)`, { images: [1, 2] }).ok).toBe(true);
    expect(checkField('description', '- ok\n![a](img:5)', { images: [1] })).toMatchObject({ ok: false, key: 'rich-image-unknown' });
  });
});
