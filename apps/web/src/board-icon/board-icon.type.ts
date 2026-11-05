/* @layer renderer-app @kind types */
import type { BrandApp } from '@drizztdourden08/tessera/brand';
import type { BOARD_ICON_NAMES } from './board-icon.constants';

type BoardIconName = (typeof BOARD_ICON_NAMES)[number];

type BoardIconSource =
  | { kind: 'brand'; app: BrandApp }
  | { kind: 'icon'; name: BoardIconName }
  | { kind: 'image'; src: string };

interface ResolvedBoardIcon {
  source: BoardIconSource;
  isDefault: boolean;
}

type IconFileResult = { ok: true; url: string } | { ok: false; reason: string };

export type { BoardIconName, BoardIconSource, IconFileResult, ResolvedBoardIcon };
