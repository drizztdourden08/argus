/* @layer renderer-app @kind types */
import type { SegmentOption } from '@drizztdourden08/tessera';

type BoardOption = SegmentOption<string>;

interface BoardSwitcherProps {
  boardKey: string | null;
}

export type { BoardOption, BoardSwitcherProps };
