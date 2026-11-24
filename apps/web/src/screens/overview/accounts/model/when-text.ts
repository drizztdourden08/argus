/* @layer renderer-app @kind logic */
import type { NextWhen } from '../../../../data/types';
import { WHEN_UNKNOWN } from '../next.constants';
import { clockText } from './clock-text';

const whenText = (when: NextWhen | undefined): string => {
  if (when?.kind === 'percent') return `At ${when.percent}% used`;
  if (when?.kind === 'time') return `At ${clockText(when.atMs)}`;
  return WHEN_UNKNOWN;
};

export { whenText };
