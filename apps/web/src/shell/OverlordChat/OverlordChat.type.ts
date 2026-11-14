/* @layer renderer-app @kind types */
import type { ActionState } from '../../data/overlord';
import type { TalkRow, TranscriptRow } from './model';

interface TranscriptViewProps {
  rows: TranscriptRow[];
  problem: string | null;
}

interface ChatBoxProps {
  running: boolean;
  action: ActionState;
  onSend: (text: string) => Promise<boolean>;
}

interface TalkRowViewProps {
  row: TalkRow;
}

interface ToolsRowProps {
  names: string[];
}

interface ChatOpen {
  open: boolean;
  toggle: () => void;
  close: () => void;
}

export type { ChatBoxProps, ChatOpen, TalkRowViewProps, ToolsRowProps, TranscriptViewProps };
