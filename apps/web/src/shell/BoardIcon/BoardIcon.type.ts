/* @layer renderer-app @kind types */
import type { BoardIconSource } from '../../board-icon';

type BoardIconSize = 'sm' | 'md' | 'lg';

interface BoardIconProps {
  source?: BoardIconSource;
  boardKey?: string;
  icon?: string | null;
  avatar?: string | null;
  cardIcon?: string;
  size?: BoardIconSize;
  title?: string;
}

export type { BoardIconProps, BoardIconSize };
