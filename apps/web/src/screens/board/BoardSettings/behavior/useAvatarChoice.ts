/* @layer renderer-app @kind logic */
import type { BrandApp } from '@drizztdourden08/tessera/brand';
import { useCallback } from 'react';
import { BRAND_PREFIX, fileToAvatar } from '../../../../avatar';
import { setAvatar } from '../../../../data/posts';
import { useDoc } from '../../../../data/react';
import type { BoardMeta } from '../../../../data/types';
import { AVATAR_REFUSED, AVATAR_SAVED } from '../BoardSettings.constants';
import type { AvatarChoice } from '../BoardSettings.type';
import { useSettingSave } from './useSettingSave';

const useAvatarChoice = (boardKey: string): AvatarChoice => {
  const setting = useDoc<BoardMeta>(boardKey, 'meta', 'board').doc?.avatar ?? null;
  const { busy, problem, setProblem, save } = useSettingSave({ boardKey, post: setAvatar, saved: AVATAR_SAVED });
  const pickBrand = useCallback((app: BrandApp) => save(`${BRAND_PREFIX}${app}`), [save]);
  const reset = useCallback(() => save(null), [save]);
  const upload = useCallback(async (files: File[]) => {
    if (!files[0]) return;
    try {
      await save(await fileToAvatar(files[0]));
    } catch (error) {
      setProblem(error instanceof Error ? error.message : 'That image could not be used.');
    }
  }, [save, setProblem]);
  const reject = useCallback(() => setProblem(AVATAR_REFUSED), [setProblem]);
  return { setting, busy, problem, pickBrand, upload, reject, reset };
};

export { useAvatarChoice };
