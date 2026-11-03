/* @layer renderer-app @kind logic */
import { toast } from '@drizztdourden08/tessera';
import type { BrandApp } from '@drizztdourden08/tessera/brand';
import { useCallback, useState } from 'react';
import { BRAND_PREFIX, fileToAvatar } from '../../../../avatar';
import { writeErrorText } from '../../../../data/api';
import { setAvatar } from '../../../../data/posts';
import { useDoc } from '../../../../data/react';
import type { BoardMeta } from '../../../../data/types';
import { postVerdict } from '../../tabs/tests/model';
import { AVATAR_SAVED } from '../BoardMore.constants';
import type { AvatarChoice } from '../BoardMore.type';

const useAvatarChoice = (boardKey: string, onClose: () => void): AvatarChoice => {
  const setting = useDoc<BoardMeta>(boardKey, 'meta', 'board').doc?.avatar ?? null;
  const [busy, setBusy] = useState(false);
  const [problem, setProblem] = useState<string | null>(null);
  const save = useCallback(async (value: string | null) => {
    setBusy(true);
    try {
      const verdict = postVerdict(await setAvatar(boardKey, value));
      setProblem(verdict);
      if (!verdict) {
        toast({ variant: 'success', message: AVATAR_SAVED });
        onClose();
      }
    } catch (error) {
      setProblem(writeErrorText(error));
    } finally {
      setBusy(false);
    }
  }, [boardKey, onClose]);
  const pickBrand = useCallback((app: BrandApp) => save(`${BRAND_PREFIX}${app}`), [save]);
  const reset = useCallback(() => save(null), [save]);
  const upload = useCallback(async (files: File[]) => {
    if (!files[0]) return;
    try {
      await save(await fileToAvatar(files[0]));
    } catch (error) {
      setProblem(error instanceof Error ? error.message : 'That image could not be used.');
    }
  }, [save]);
  return { setting, busy, problem, pickBrand, upload, reset };
};

export { useAvatarChoice };
