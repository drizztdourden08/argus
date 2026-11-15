/* @layer renderer-app @kind types */
import type { ActionState } from '../../data/overlord';

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

interface ChatBoxProps {
  running: boolean;
  action: ActionState;
  onSend: (text: string) => Promise<boolean>;
}

interface ChatOpen {
  open: boolean;
  toggle: () => void;
  close: () => void;
}

export type {
  ChatBoxProps,
  ChatOpen,
  ContentBlock,
  Piece,
  TranscriptRow,
};
