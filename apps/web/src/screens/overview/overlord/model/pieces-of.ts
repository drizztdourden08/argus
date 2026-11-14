/* @layer renderer-app @kind logic */
import type { TranscriptEvent } from '../../../../data/overlord';
import { OWNER_PREFIX, OWNER_SPLIT } from './model.constants';
import type { ContentBlock, Piece } from './model.type';

const blocksOf = (content: unknown): ContentBlock[] => {
  if (typeof content === 'string') return [{ type: 'text', text: content }];
  return Array.isArray(content) ? (content as ContentBlock[]) : [];
};

const userPieces = (text: string): Piece[] =>
  text
    .split(OWNER_SPLIT)
    .filter((part) => part.trim() !== '')
    .map((part) => (part.startsWith(OWNER_PREFIX) ? { kind: 'owner', text: part.slice(OWNER_PREFIX.length) } : { kind: 'wake', text: part }));

const assistantPiece = (block: ContentBlock): Piece[] => {
  if (block.type === 'tool_use') return [{ kind: 'tool', name: block.name ?? 'tool' }];
  return block.type === 'text' && block.text?.trim() ? [{ kind: 'assistant', text: block.text }] : [];
};

const userPiecesOf = (block: ContentBlock): Piece[] => (block.type === 'text' && block.text ? userPieces(block.text) : []);

const piecesOf = (event: TranscriptEvent | string): Piece[] => {
  if (typeof event === 'string') return [];
  const blocks = blocksOf(event.message?.content);
  if (event.type === 'assistant') return blocks.flatMap(assistantPiece);
  return event.type === 'user' ? blocks.flatMap(userPiecesOf) : [];
};

export { piecesOf };
