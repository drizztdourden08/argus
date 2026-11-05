/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { OWNER_DEFAULTS, resolveOwnerFace } from '../../src/owner';

const PNG = 'data:image/png;base64,AAAA';
const profile = (avatar: string | null) => ({ ...OWNER_DEFAULTS, avatar });

describe('I-111 the face of the owner', { tags: ['critical'] }, () => {
  it('is the user icon by default', () => {
    expect(resolveOwnerFace(profile(null))).toEqual({ kind: 'default' });
  });

  it('reads initials, a brand, an image and the served avatar url', () => {
    expect(resolveOwnerFace({ name: 'Ann', initials: 'AN', avatar: 'initials' })).toEqual({ kind: 'initials', initials: 'AN' });
    expect(resolveOwnerFace(profile('brand:rotp'))).toEqual({ kind: 'brand', app: 'rotp' });
    expect(resolveOwnerFace(profile(PNG))).toEqual({ kind: 'image', src: PNG });
    expect(resolveOwnerFace(profile('/api/owner/avatar?v=0a1b2c3d'))).toEqual({ kind: 'image', src: '/api/owner/avatar?v=0a1b2c3d' });
  });

  it('falls back to the default for anything else', () => {
    for (const bad of ['brand:nope', 'data:image/jpeg;base64,AAAA', 'https://example.com/a.png', '/api/owner/avatar', '']) {
      expect(resolveOwnerFace(profile(bad))).toEqual({ kind: 'default' });
    }
  });
});
