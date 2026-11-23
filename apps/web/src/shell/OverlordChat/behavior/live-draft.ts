/* @layer renderer-app @kind logic */
import type { StreamInner, TranscriptLine } from '../../../data/overlord';
import { FINAL_EVENTS } from '../OverlordChat.constants';
import type { DraftBlock, LiveDraft, TalkRow } from '../OverlordChat.type';

const endsTurn = (line: TranscriptLine): boolean => typeof line.event !== 'string' && FINAL_EVENTS.includes(line.event.type ?? '');

const innerOf = (line: TranscriptLine): StreamInner | null => {
  const event = line.event;
  if (typeof event === 'string' || event.type !== 'stream_event') return null;
  return typeof event.event === 'object' ? event.event : null;
};

const kindOfDelta = (type: string | undefined): string => (type === 'thinking_delta' ? 'thinking' : 'text');

const startBlock = (blocks: Map<number, DraftBlock>, inner: StreamInner): void => {
  const start = inner.content_block;
  blocks.set(inner.index ?? 0, { kind: start?.type ?? 'text', name: start?.name ?? null, text: '' });
};

const growBlock = (blocks: Map<number, DraftBlock>, inner: StreamInner): void => {
  const at = inner.index ?? 0;
  const delta = inner.delta;
  if (!delta) return;
  const block = blocks.get(at) ?? { kind: kindOfDelta(delta.type), name: null, text: '' };
  block.text += delta.thinking ?? delta.text ?? '';
  blocks.set(at, block);
};

const apply = (blocks: Map<number, DraftBlock>, inner: StreamInner): void => {
  if (inner.type === 'message_start') blocks.clear();
  if (inner.type === 'content_block_start') startBlock(blocks, inner);
  if (inner.type === 'content_block_delta') growBlock(blocks, inner);
};

const rowOf = ([at, block]: [number, DraftBlock]): TalkRow[] => {
  if (block.text.trim() === '' || (block.kind !== 'thinking' && block.kind !== 'text')) return [];
  const kind = block.kind === 'thinking' ? 'thinking' : 'assistant';
  return [{ kind, key: `live-${kind}-${at}`, text: block.text.trim(), live: true }];
};

const liveDraft = (lines: readonly TranscriptLine[]): LiveDraft => {
  const last = lines.findLastIndex(endsTurn);
  const blocks = new Map<number, DraftBlock>();
  lines.slice(last + 1).forEach((line) => {
    const inner = innerOf(line);
    if (inner) apply(blocks, inner);
  });
  const ordered = [...blocks.entries()].sort((a, b) => a[0] - b[0]);
  const running = ordered.findLast(([, block]) => block.kind === 'tool_use');
  return { rows: ordered.flatMap(rowOf), tool: running?.[1].name ?? null };
};

export { liveDraft };
