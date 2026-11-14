/* @layer renderer-app @kind types */
type TalkKind = 'assistant' | 'owner' | 'wake';

interface TalkRow {
  kind: TalkKind;
  key: string;
  text: string;
}

interface ToolsRow {
  kind: 'tools';
  key: string;
  names: string[];
}

type TranscriptRow = TalkRow | ToolsRow;

interface ContentBlock {
  type?: string;
  text?: string;
  name?: string;
}

type Piece = Omit<TalkRow, 'key'> | { kind: 'tool'; name: string };

export type { ContentBlock, Piece, TalkRow, TranscriptRow };
