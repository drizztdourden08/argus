/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { BOARD_ICON_NAMES, ICON_MAX, resolveBoardIcon } from '../../src/board-icon';

const PNG = 'data:image/png;base64,AAAA';
const SVG = 'data:image/svg+xml;base64,AAAA';

describe('I-109 the icon of a board', { tags: ['critical'] }, () => {
  it('reads brand, curated icon, png, svg and card url settings', () => {
    expect(resolveBoardIcon('x', 'brand:rotp').source).toEqual({ kind: 'brand', app: 'rotp' });
    expect(resolveBoardIcon('x', 'icon:package').source).toEqual({ kind: 'icon', name: 'package' });
    expect(resolveBoardIcon('x', PNG).source).toEqual({ kind: 'image', src: PNG });
    expect(resolveBoardIcon('x', SVG).source).toEqual({ kind: 'image', src: SVG });
    const url = '/api/p/x/icon?v=0a1b2c3d';
    expect(resolveBoardIcon('x', null, null, url).source).toEqual({ kind: 'image', src: url });
    expect(resolveBoardIcon('x', 'icon:zap').isDefault).toBe(false);
  });

  it('prefers the board setting over the card icon', () => {
    expect(resolveBoardIcon('x', 'icon:leaf', null, 'brand:brock').source).toEqual({ kind: 'icon', name: 'leaf' });
  });

  it('refuses unknown names, other types, foreign urls and oversize data', () => {
    for (const bad of ['icon:nope', 'brand:nope', 'data:image/jpeg;base64,AAAA', 'https://example.com/a.png', '/api/p/x/icon', 7, '']) {
      expect(resolveBoardIcon('x', bad).isDefault).toBe(true);
    }
    expect(resolveBoardIcon('x', `${PNG}${'A'.repeat(ICON_MAX)}`).isDefault).toBe(true);
  });

  it('defaults to the key brand, then a brand avatar, then a stable curated icon', () => {
    expect(resolveBoardIcon('pixelarttool')).toEqual({ source: { kind: 'brand', app: 'bxel' }, isDefault: true });
    expect(resolveBoardIcon('other', null, 'brand:tessera').source).toEqual({ kind: 'brand', app: 'tessera' });
    expect(resolveBoardIcon('other', null, PNG).source.kind).toBe('icon');
    const first = resolveBoardIcon('some-board').source;
    expect(resolveBoardIcon('some-board').source).toEqual(first);
    expect(first.kind === 'icon' && BOARD_ICON_NAMES.includes(first.name)).toBe(true);
    expect(resolveBoardIcon('toString').isDefault).toBe(true);
  });
});
