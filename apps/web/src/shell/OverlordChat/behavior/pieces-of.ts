/* @layer renderer-app @kind logic */
import type { TranscriptEvent } from '../../../data/overlord';
import { formatDuration } from '../../../format';
import { OWNER_PREFIX, OWNER_SPLIT } from '../OverlordChat.constants';
import type { ContentBlock, Piece } from '../OverlordChat.type';
import { approvalPiece } from './approval-piece';
import { resultText } from './result-text';
import { toolSummary } from './tool-summary';

const blocksOf = (content: unknown): ContentBlock[] => {
  if (typeof content === 'string') return [{ type: 'text', text: content }];
  return Array.isArray(content) ? (content as ContentBlock[]) : [];
};

const userPieces = (text: string, source?: string): Piece[] =>
  text
    .split(OWNER_SPLIT)
    .filter((part) => part.trim() !== '')
    .map((part) => (part.startsWith(OWNER_PREFIX) ? { kind: 'owner', text: part.slice(OWNER_PREFIX.length) } : { kind: source === 'rule' ? 'rule' : 'wake', text: part }));

const assistantPiece = (block: ContentBlock): Piece[] => {
  if (block.type === 'tool_use') return [{ kind: 'tool', id: block.id ?? '', name: block.name ?? 'tool', summary: toolSummary(block.input) }];
  if (block.type === 'thinking') return block.thinking?.trim() ? [{ kind: 'thinking', text: block.thinking.trim() }] : [];
  return block.type === 'text' && block.text?.trim() ? [{ kind: 'assistant', text: block.text }] : [];
};

const userPiecesOf =
  (source?: string) =>
  (block: ContentBlock): Piece[] => {
    if (block.type === 'tool_result') return [{ kind: 'result', id: block.tool_use_id ?? '', failed: block.is_error === true, text: resultText(block.content) }];
    return block.type === 'text' && block.text ? userPieces(block.text, source) : [];
  };

const turnPiece = (event: TranscriptEvent): Piece[] => [
  { kind: 'turn', text: event.duration_ms === undefined ? 'Turn finished' : `Turn finished in ${formatDuration(event.duration_ms)}` },
];

const piecesOf = (event: TranscriptEvent | string): Piece[] => {
  if (typeof event === 'string') return [];
  if (event.type === 'result') return turnPiece(event);
  if (event.type === 'argus') return event.event === 'approval' ? approvalPiece(event) : [];
  const blocks = blocksOf(event.message?.content);
  if (event.type === 'assistant') return blocks.flatMap(assistantPiece);
  return event.type === 'user' ? blocks.flatMap(userPiecesOf(event.argus?.source)) : [];
};

export { piecesOf };
