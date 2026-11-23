/* @layer renderer-app @kind logic */
import type { SwitchSyncDoc } from '../../../../../data/types';
import { COUNT_KEYS } from '../switch.constants';

const countsText = (counts: SwitchSyncDoc['counts']): string =>
  COUNT_KEYS.filter((key) => typeof counts?.[key] === 'number')
    .map((key) => `${counts?.[key] ?? 0} ${key}`)
    .join(', ');

export { countsText };
