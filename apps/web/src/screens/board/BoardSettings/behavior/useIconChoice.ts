/* @layer renderer-app @kind logic */
import { useCallback } from 'react';
import { fileToIcon } from '../../../../board-icon';
import { setIcon } from '../../../../data/posts';
import { useDoc } from '../../../../data/react';
import type { BoardMeta } from '../../../../data/types';
import { ICON_REFUSED, ICON_SAVED } from '../BoardSettings.constants';
import type { IconChoice } from '../BoardSettings.type';
import { useSavePicks } from './useSavePicks';
import { useSettingSave } from './useSettingSave';

const useIconChoice = (boardKey: string): IconChoice => {
  const meta = useDoc<BoardMeta>(boardKey, 'meta', 'board').doc;
  const { busy, problem, setProblem, save } = useSettingSave({ boardKey, post: setIcon, saved: ICON_SAVED });
  const { pickIcon, reset } = useSavePicks(save);
  const upload = useCallback(async (files: File[]) => {
    if (!files[0]) return;
    const result = await fileToIcon(files[0]);
    if (result.ok) await save(result.url);
    else setProblem(result.reason);
  }, [save, setProblem]);
  const reject = useCallback(() => setProblem(ICON_REFUSED), [setProblem]);
  return { setting: meta?.icon ?? null, avatar: meta?.avatar ?? null, busy, problem, pickIcon, upload, reject, reset };
};

export { useIconChoice };
