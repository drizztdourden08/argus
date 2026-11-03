/* @layer renderer-app @kind types */
import type { DropZoneStatus } from '@drizztdourden08/tessera';
import type { RunAttachment, RunEntry } from '../../../data/runs';

interface OpenLog {
  onLog: (runId: string) => void;
}

interface LogButtonProps extends OpenLog {
  run: RunEntry | null;
}

interface RunTerminalProps extends OpenLog {
  run: RunEntry;
  boardKey: string;
}

interface TerminalBodyProps {
  run: RunEntry;
  boardKey: string;
  limit: number;
  height: number;
}

interface RunStopButtonProps {
  run: RunEntry;
  boardKey: string;
}

interface RunLogDrawerProps {
  run: RunEntry | null;
  boardKey: string;
  onClose: () => void;
}

interface RunAttachmentsProps {
  boardKey: string;
  runId: string;
}

interface RunAttachments {
  items: RunAttachment[];
  uploading: boolean;
  status: DropZoneStatus | undefined;
  attach: (file: File, caption: string) => Promise<boolean>;
}

interface TerminalInput {
  enabled: boolean;
  placeholder: string;
}

export type {
  LogButtonProps, RunAttachments, RunAttachmentsProps, RunLogDrawerProps, RunStopButtonProps, RunTerminalProps, TerminalBodyProps, TerminalInput,
};
