/* @layer renderer-app @kind logic */
import { useMemo } from 'react';
import { STATUS_POLL_MS, readStatus, sendMessage, useAction, useOverlordTranscript } from '../../../data/overlord';
import { useRemoteRead } from '../../../data/remote';
import { isRunning, toRows } from '../model';

const noop = (): void => {};

const useChatFeed = () => {
  const status = useRemoteRead(readStatus, STATUS_POLL_MS);
  const feed = useOverlordTranscript(status.data?.sessionId ?? null);
  const chat = useAction(noop);
  const rows = useMemo(() => toRows(feed.lines), [feed.lines]);
  return {
    rows,
    problem: feed.problem,
    running: isRunning(status.data?.status),
    action: chat,
    onSend: (text: string) => chat.run(() => sendMessage(text)),
  };
};

export { useChatFeed };
