/* @layer renderer-app @kind logic */
import { useEffect } from 'react';
import { useStreams } from '../../../data/react/behavior/useStreams';

const WARM_COLLECTIONS = ['meta', 'threads'] as const;

const noop = (): void => undefined;

const useBoardWarm = (key: string | null): void => {
  const streams = useStreams();
  useEffect(() => {
    if (!key) return undefined;
    const stream = streams.get(key);
    const leaves = WARM_COLLECTIONS.map((coll) => stream.subscribe(coll, noop));
    return () => {
      for (const leave of leaves) leave();
    };
  }, [streams, key]);
};

export { useBoardWarm };
