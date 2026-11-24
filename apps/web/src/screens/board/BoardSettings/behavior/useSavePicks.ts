/* @layer renderer-app @kind logic */
import { useCallback } from 'react';
import type { BoardIconName } from '../../../../board-icon';
import { ICON_VALUE_PREFIX } from '../BoardSettings.constants';
import type { SettingSave } from '../BoardSettings.type';

const useSavePicks = (save: SettingSave['save']) => {
  const pickIcon = useCallback((name: BoardIconName) => save(`${ICON_VALUE_PREFIX}${name}`), [save]);
  const reset = useCallback(() => save(null), [save]);
  return { pickIcon, reset };
};

export { useSavePicks };
