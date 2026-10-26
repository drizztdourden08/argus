/* @layer renderer-app @kind logic */
import { toast } from '@drizztdourden08/tessera';
import { useCallback, useRef, useState } from 'react';
import { ownerEntry, postEntry } from '../../../data/posts';
import { failureToast } from './failure-toast';
import type { OwnerPoster, PostSpec } from './owner-posts.type';
import { verdictToast } from './verdict-toast';

const useOwnerPost = (boardKey: string): OwnerPoster => {
  const flying = useRef(new Set<string>());
  const [busy, setBusy] = useState<ReadonlySet<string>>(new Set());
  const sync = useCallback(() => setBusy(new Set(flying.current)), []);
  const submit = useCallback(
    async (spec: PostSpec): Promise<boolean> => {
      if (flying.current.has(spec.key)) return false;
      flying.current.add(spec.key);
      sync();
      try {
        const result = await postEntry(boardKey, ownerEntry(spec.fields));
        toast(verdictToast(result, spec.said));
        return result.status !== 'conflict' && result.status !== 'refused';
      } catch (error) {
        toast(failureToast(error));
        return false;
      } finally {
        flying.current.delete(spec.key);
        sync();
      }
    },
    [boardKey, sync],
  );
  const isBusy = useCallback((key: string) => busy.has(key), [busy]);
  return { isBusy, submit };
};

export { useOwnerPost };
