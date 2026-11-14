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

interface ControlSet {
  start: boolean;
  restart: boolean;
  stop: boolean;
}

interface ContentBlock {
  type?: string;
  text?: string;
  name?: string;
}

interface SpendLook {
  ratio: number;
  tone: 'success' | 'warning' | 'danger';
}

type Piece = Omit<TalkRow, 'key'> | { kind: 'tool'; name: string };

export type { ContentBlock, ControlSet, Piece, SpendLook, TalkKind, TalkRow, ToolsRow, TranscriptRow };
