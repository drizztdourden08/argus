/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { imageMarkdown, itemImages, mergeImages, picturesOf, shotsOf } from '../../src/data/media';
import { gridItems, itemCounts, viewerItems } from '../../src/screens/board/tabs/media/model';

const image = (n: number, caption = `shot ${n}`, at?: string) => ({ n, url: `/files/alpha/proofs/I-1-${n}.png`, caption, at });

describe('P8 media collection model', { tags: ['critical'] }, () => {
  const items = [
    { id: 'I-1', title: 'First', images: [image(1, 'old', '2026-11-01T10:00:00.000Z'), image(2, 'newer', '2026-11-02T10:00:00.000Z')] },
    { id: 'I-2', title: 'Second', images: [image(1, 'newest', '2026-11-03T10:00:00.000Z')] },
    { id: 'I-3', title: 'Trashed', deletedAt: '2026-11-03T11:00:00.000Z', images: [image(1)] },
    { id: 'I-4', title: 'Bare' },
  ];

  it('lists every image of every live item, newest first, each with its item as the source', () => {
    const shots = shotsOf(items);
    expect(shots.map((shot) => shot.key)).toEqual(['I-2:1', 'I-1:2', 'I-1:1']);
    expect(shots[0]).toMatchObject({ item: 'I-2', title: 'Second', caption: 'newest' });
  });

  it('the grid and the viewer carry the item as the source and the name', () => {
    const shots = shotsOf(items);
    expect(gridItems(shots).map((cell) => cell.source)).toEqual(['I-2', 'I-1', 'I-1']);
    expect(viewerItems(shots)[0]).toMatchObject({ kind: 'image', src: '/files/alpha/proofs/I-1-1.png', name: 'I-2: newest' });
  });

  it('counts the images per item in id order', () => {
    expect(itemCounts(shotsOf(items))).toEqual([
      { id: 'I-1', title: 'First', count: 2 },
      { id: 'I-2', title: 'Second', count: 1 },
    ]);
  });

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
