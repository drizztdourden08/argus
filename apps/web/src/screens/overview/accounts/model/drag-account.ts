/* @layer renderer-app @kind logic */
import type { DragEvent } from 'react';
import { DRAG_TYPE } from '../next.constants';

const dragAccount = (key: string) => (event: DragEvent<HTMLElement>): void => {
  event.dataTransfer.setData(DRAG_TYPE, key);
};

export { dragAccount };
