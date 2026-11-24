/* @layer renderer-app @kind logic */
import { useCallback } from 'react';
import { fileToAvatar } from '../../../../avatar';
import { setAvatar } from '../../../../data/posts';
import { useDoc } from '../../../../data/react';
import type { BoardMeta } from '../../../../data/types';
import { AVATAR_REFUSED, AVATAR_SAVED } from '../BoardSettings.constants';
import type { AvatarChoice } from '../BoardSettings.type';
import { useSavePicks } from './useSavePicks';
import { useSettingSave } from './useSettingSave';

const useAvatarChoice = (boardKey: string): AvatarChoice => {
  const setting = useDoc<BoardMeta>(boardKey, 'meta', 'board').doc?.avatar ?? null;
  const { busy, problem, setProblem, save } = useSettingSave({ boardKey, post: setAvatar, saved: AVATAR_SAVED });
  const { pickIcon, reset } = useSavePicks(save);
  const upload = useCallback(async (files: File[]) => {
    if (!files[0]) return;
    try {
      await save(await fileToAvatar(files[0]));
    } catch (error) {
      setProblem(error instanceof Error ? error.message : 'That image could not be used.');
    }
  }, [save, setProblem]);
  const reject = useCallback(() => setProblem(AVATAR_REFUSED), [setProblem]);
  return { setting, busy, problem, pickIcon, upload, reject, reset };
};

export { useAvatarChoice };
