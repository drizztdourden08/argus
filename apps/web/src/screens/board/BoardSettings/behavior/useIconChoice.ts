/* @layer renderer-app @kind logic */
import type { BrandApp } from '@drizztdourden08/tessera/brand';
import { useCallback } from 'react';
import { BRAND_PREFIX } from '../../../../avatar';
import { fileToIcon } from '../../../../board-icon';
import { setIcon } from '../../../../data/posts';
import { useDoc } from '../../../../data/react';
import type { BoardIconName } from '../../../../board-icon';
import type { BoardMeta } from '../../../../data/types';
import { ICON_SAVED, ICON_VALUE_PREFIX } from '../BoardSettings.constants';
import type { IconChoice } from '../BoardSettings.type';
import { useSettingSave } from './useSettingSave';

const useIconChoice = (boardKey: string): IconChoice => {
  const meta = useDoc<BoardMeta>(boardKey, 'meta', 'board').doc;
  const { busy, problem, setProblem, save } = useSettingSave({ boardKey, post: setIcon, saved: ICON_SAVED });
  const pickBrand = useCallback((app: BrandApp) => save(`${BRAND_PREFIX}${app}`), [save]);
  const pickIcon = useCallback((name: BoardIconName) => save(`${ICON_VALUE_PREFIX}${name}`), [save]);
  const reset = useCallback(() => save(null), [save]);
  const upload = useCallback(async (files: File[]) => {
    if (!files[0]) return;
    const result = await fileToIcon(files[0]);
    if (result.ok) await save(result.url);
    else setProblem(result.reason);
  }, [save, setProblem]);
  return { setting: meta?.icon ?? null, avatar: meta?.avatar ?? null, busy, problem, pickBrand, pickIcon, upload, reset };
};

export { useIconChoice };
