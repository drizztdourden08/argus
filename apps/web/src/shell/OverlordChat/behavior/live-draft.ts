/* @layer renderer-app @kind logic */
import type { StreamInner, TranscriptLine } from '../../../data/overlord';
import { FINAL_EVENTS } from '../OverlordChat.constants';
import type { LiveDraft, TalkRow } from '../OverlordChat.type';

interface Block {
  kind: string;
  name: string | null;
  text: string;
}

const isFinal = (line: TranscriptLine): boolean => typeof line.event !== 'string' && FINAL_EVENTS.includes(line.event.type ?? '');

const innerOf = (line: TranscriptLine): StreamInner | null => {
  const event = line.event;
  if (typeof event === 'string' || event.type !== 'stream_event') return null;
  return typeof event.event === 'object' ? event.event : null;
};

const kindOfDelta = (type: string | undefined): string => (type === 'thinking_delta' ? 'thinking' : 'text');

const apply = (blocks: Map<number, Block>, inner: StreamInner): void => {
  const at = inner.index ?? 0;
  if (inner.type === 'message_start') blocks.clear();
  if (inner.type === 'content_block_start') {
    blocks.set(at, { kind: inner.content_block?.type ?? 'text', name: inner.content_block?.name ?? null, text: '' });
    return;
  }
  if (inner.type !== 'content_block_delta' || !inner.delta) return;
  const block = blocks.get(at) ?? { kind: kindOfDelta(inner.delta.type), name: null, text: '' };
  block.text += inner.delta.thinking ?? inner.delta.text ?? '';
  blocks.set(at, block);
};

const rowOf = ([at, block]: [number, Block]): TalkRow[] => {
  if (block.text.trim() === '' || (block.kind !== 'thinking' && block.kind !== 'text')) return [];
  const kind = block.kind === 'thinking' ? 'thinking' : 'assistant';
  return [{ kind, key: `live-${kind}-${at}`, text: block.text.trim(), live: true }];
};

const liveDraft = (lines: readonly TranscriptLine[]): LiveDraft => {
  const last = lines.findLastIndex(isFinal);
  const blocks = new Map<number, Block>();
  lines.slice(last + 1).forEach((line) => {
    const inner = innerOf(line);
    if (inner) apply(blocks, inner);
  });
  const ordered = [...blocks.entries()].sort((a, b) => a[0] - b[0]);
  const running = ordered.findLast(([, block]) => block.kind === 'tool_use');
  return { rows: ordered.flatMap(rowOf), tool: running?.[1].name ?? null };
};

export { liveDraft };
