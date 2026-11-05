/* @layer renderer-app @kind types */
import type { SegmentOption } from '@drizztdourden08/tessera';
import type { Card } from '../../data/types';

type BoardOption = SegmentOption;

interface BoardSwitcherProps {
  boardKey: string | null;
}

interface BoardOptionIconProps {
  card: Card;
}

export type { BoardOption, BoardOptionIconProps, BoardSwitcherProps };
