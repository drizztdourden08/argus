/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { imageMarkdown, itemImages, mergeImages, picturesOf } from '../../src/data/media';

const image = (n: number, caption = `shot ${n}`, at?: string) => ({ n, url: `/files/alpha/proofs/I-1-${n}.png`, caption, at });

describe('P8 media model', { tags: ['critical'] }, () => {
  it('keeps only well formed images and merges a fresh upload by its number', () => {
    expect(itemImages({ images: [{ n: 1, url: '/a.png' }, { url: '/b.png' }, { n: 3 }] })).toEqual([{ n: 1, url: '/a.png', caption: '', at: undefined }]);
    const merged = mergeImages([image(2)], [image(3), image(2, 'again')]);
    expect(merged.map((one) => one.n)).toEqual([2, 3]);
    expect(merged[0]?.caption).toBe('shot 2');
  });

  it('resolves img:n to the image url and nothing else', () => {
    const pictures = picturesOf([image(1), image(4)]);
    expect(pictures.resolve('img:4')).toBe('/files/alpha/proofs/I-1-4.png');
    for (const src of ['img:9', 'https://example.com/a.png', 'img:x']) expect(pictures.resolve(src)).toBeUndefined();
    expect(pictures.gallery.map((item) => item.id)).toEqual(['1', '4']);
  });

  it('writes the image line the rich rule reads, with a safe caption', () => {
    expect(imageMarkdown({ n: 3, caption: 'the [form] (v2)\nlobby' })).toBe('![the form v2 lobby](img:3)');
    expect(imageMarkdown({ n: 3, caption: '  ' })).toBe('![screenshot](img:3)');
  });
});
