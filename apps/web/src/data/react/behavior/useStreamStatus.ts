/* @layer renderer-app @kind logic */
import { useSyncExternalStore } from 'react';
import type { StreamStatus } from '../../stream';
import { useStreams } from './useStreams';

const useStreamStatus = (scope: string): StreamStatus => {
  const stream = useStreams().get(scope);
  return useSyncExternalStore(stream.subscribeStatus, () => stream.getStatus());
};

export { useStreamStatus };
