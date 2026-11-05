/* @layer renderer-app @kind logic */
import { toast } from '@drizztdourden08/tessera';
import { useCallback, useState } from 'react';
import { writeErrorText } from '../../../../data/api';
import { postVerdict } from '../../tabs/tests/model';
import type { SettingSave, SettingSaveOptions } from '../BoardSettings.type';

const useSettingSave = (options: SettingSaveOptions): SettingSave => {
  const { boardKey, post, saved } = options;
  const [busy, setBusy] = useState(false);
  const [problem, setProblem] = useState<string | null>(null);
  const save = useCallback(async (value: string | null) => {
    setBusy(true);
    try {
      const verdict = postVerdict(await post(boardKey, value));
      setProblem(verdict);
      if (!verdict) toast({ variant: 'success', message: saved });
    } catch (error) {
      setProblem(writeErrorText(error));
    } finally {
      setBusy(false);
    }
  }, [boardKey, post, saved]);
  return { busy, problem, setProblem, save };
};

export { useSettingSave };
