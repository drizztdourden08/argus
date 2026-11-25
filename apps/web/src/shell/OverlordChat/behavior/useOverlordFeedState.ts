/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import { useOverlordTranscript } from '../../../data/overlord';
import type { OverlordFeedApi } from '../OverlordChat.type';
import { useOverlordStatus } from './useOverlordStatus';

const useOverlordFeedState = (): OverlordFeedApi => {
  const status = useOverlordStatus();
  const feed = useOverlordTranscript(status.data?.sessionId ?? null);
  return useMemo(() => ({ status: status.data, lines: feed.lines, problem: feed.problem }), [status.data, feed.lines, feed.problem]);
};

export { useOverlordFeedState };
