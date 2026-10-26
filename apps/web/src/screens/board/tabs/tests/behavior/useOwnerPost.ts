/* @layer renderer-app @kind logic */
import { toast } from '@drizztdourden08/tessera';
import { useCallback, useRef, useState } from 'react';
import type { PostResult } from '../../../../../data/api';
import { postVerdict, writeErrorText } from '../model';
import type { OwnerPost } from '../tests.type';

const useOwnerPost = (): OwnerPost => {
  const [busy, setBusy] = useState(false);
  const flying = useRef(false);
  const run = useCallback(async (send: () => Promise<PostResult>, done: string): Promise<string | null> => {
    if (flying.current) return 'Still posting. Wait a moment.';
    flying.current = true;
    setBusy(true);
    try {
      const verdict = postVerdict(await send());
      if (verdict) return verdict;
      toast({ variant: 'success', message: done });
      return null;
    } catch (error) {
      return writeErrorText(error);
    } finally {
      flying.current = false;
      setBusy(false);
    }
  }, []);
  return { busy, run };
};

export { useOwnerPost };
