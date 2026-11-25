/* @layer renderer-app @kind logic */
import { useMemo, useRef, useState } from 'react';
import { sendMessage, useAction } from '../../../data/overlord';
import type { PendingSend } from '../OverlordChat.type';
import { useOverlordFeed } from './useOverlordFeed';
import { isRunning } from './is-running';
import { overlordNote } from './overlord-note';
import { liveDraft } from './live-draft';
import { workingActivity } from './working-activity';
import { countOf } from './count-of';
import { queuedMessages } from './queued-messages';
import { unsettled } from './unsettled';
import { toMessages } from './to-messages';
import { toRows } from './to-rows';

const useChatFeed = (held: readonly string[], reload: () => void) => {
  const feed = useOverlordFeed();
  const { status } = feed;
  const chat = useAction(reload);
  const [pending, setPending] = useState<PendingSend[]>([]);
  const next = useRef(0);
  const rows = useMemo(() => toRows(feed.lines), [feed.lines]);
  const draft = useMemo(() => liveDraft(feed.lines), [feed.lines]);
  const messages = useMemo(() => toMessages([...rows, ...draft.rows]), [rows, draft]);
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
    working: status?.status === 'turn' ? workingActivity(rows, draft) : null,
    running: isRunning(status?.status),
    note: overlordNote(status),
    action: chat,
    onSend,
  };
};

export { useChatFeed };
