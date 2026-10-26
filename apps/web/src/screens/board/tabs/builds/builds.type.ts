/* @layer renderer-app @kind types */
import type { DropZoneStatus } from '@drizztdourden08/tessera';
import type { RunAttachment, RunEntry } from '../../../../data/runs';
import type { LauncherViews, RunGroups } from './model';

interface BuildsData {
  ready: boolean;
  views: LauncherViews;
  groups: RunGroups;
  runs: RunEntry[];
}

interface RetiredFlag {
  shown: boolean;
  set: (open: boolean) => void;
}

interface RunAttachments {
  items: RunAttachment[];
  uploading: boolean;
  status: DropZoneStatus | undefined;
  attach: (file: File, caption: string) => Promise<boolean>;
}

export type { BuildsData, RetiredFlag, RunAttachments };
