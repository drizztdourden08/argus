/* @layer renderer-app @kind logic */
import { useMemo, useRef, useState } from 'react';
import { STATUS_POLL_MS, readStatus, sendMessage, useAction, useOverlordTranscript } from '../../../data/overlord';
import { useRemoteRead } from '../../../data/remote';
import type { PendingSend } from '../OverlordChat.type';
import { isRunning } from './is-running';
import { overlordNote } from './overlord-note';
import { countOf } from './count-of';
import { queuedMessages } from './queued-messages';
import { unsettled } from './unsettled';
import { toMessages } from './to-messages';
import { toRows } from './to-rows';

const useChatFeed = (held: readonly string[], reload: () => void) => {
  const status = useRemoteRead(readStatus, STATUS_POLL_MS);
  const feed = useOverlordTranscript(status.data?.sessionId ?? null);
  const chat = useAction(reload);
  const [pending, setPending] = useState<PendingSend[]>([]);
  const next = useRef(0);
  const rows = useMemo(() => toRows(feed.lines), [feed.lines]);
  const messages = useMemo(() => toMessages(rows), [rows]);
  const visible = unsettled(pending, rows, held);

  const onSend = async (text: string): Promise<boolean> => {
    next.current += 1;
    const send: PendingSend = { id: next.current, text, baseline: countOf(text, rows, held), state: 'sending' };
    setPending((all) => [...all, send]);
    const sent = await chat.run(() => sendMessage(text));
    const keep = (all: PendingSend[]) => (sent ? all.map((one) => (one.id === send.id ? { ...one, state: 'sent' as const } : one)) : all.filter((one) => one.id !== send.id));
    setPending(keep);
    return sent;
  };

  return {
    messages,
    queued: queuedMessages(held, visible),
    problem: feed.problem,
    running: isRunning(status.data?.status),
    note: overlordNote(status.data),
    action: chat,
    onSend,
  };
};

export { useChatFeed };
