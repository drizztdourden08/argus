/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { fileToAvatar, resolveAvatar } from '../../src/avatar';

describe('I-83 the avatar of a lead', { tags: ['critical'] }, () => {
  it('defaults to the brand mark of the app a board key belongs to', () => {
    expect(resolveAvatar('tessera')).toEqual({ kind: 'brand', app: 'tessera' });
    expect(resolveAvatar('pixelarttool')).toEqual({ kind: 'brand', app: 'bxel' });
    expect(resolveAvatar('argus')).toEqual({ kind: 'brand', app: 'argus' });
  });

  it('has no source for a board without a brand, so the initials show', () => {
    expect(resolveAvatar('somethingelse')).toBeNull();
    expect(resolveAvatar('toString')).toBeNull();
  });

  it('prefers the setting, a brand or an image, and ignores a bad one', () => {
    expect(resolveAvatar('brock', 'brand:rotp')).toEqual({ kind: 'brand', app: 'rotp' });
    expect(resolveAvatar('other', 'data:image/png;base64,AAAA')).toEqual({ kind: 'image', src: 'data:image/png;base64,AAAA' });
    expect(resolveAvatar('brock', 'brand:nope')).toEqual({ kind: 'brand', app: 'brock' });
    expect(resolveAvatar('other', 'https://example.com/x.png')).toBeNull();
    expect(resolveAvatar('other', 7)).toBeNull();
  });

  it('reads an image file into a data URL and refuses a wrong type or a big one', async () => {
    const small = new File([new Uint8Array([137, 80, 78, 71])], 'a.png', { type: 'image/png' });
    expect(await fileToAvatar(small)).toMatch(/^data:image\/png;base64,/);
    await expect(fileToAvatar(new File(['x'], 'a.svg', { type: 'image/svg+xml' }))).rejects.toThrow('Only png');
    await expect(fileToAvatar(new File([new Uint8Array(200 * 1024)], 'b.png', { type: 'image/png' }))).rejects.toThrow('over 180 KB');
  });
});
