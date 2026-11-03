/* @layer renderer-app @kind types */
import type { Shot } from '../../../../data/media';

interface MediaTabProps {
  boardKey: string;
}

interface ItemLinksProps {
  boardKey: string;
  shots: readonly Shot[];
}

interface ItemCount {
  id: string;
  title: string;
  count: number;
}

export type { ItemCount, ItemLinksProps, MediaTabProps };
