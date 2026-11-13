/* @layer renderer-app @kind logic */
import { useState } from 'react';
import { LOG_START, LOG_STEP } from '../OwnerFeed.constants';
import type { LogWindow } from './log.type';

const useLogWindow = (total: number): LogWindow => {
  const [shown, setShown] = useState(LOG_START);
  return {
    shown,
    left: Math.max(0, total - shown),
    more: () => setShown((current) => current + LOG_STEP),
    less: () => setShown(LOG_START),
  };
};

export { useLogWindow };
