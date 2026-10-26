/* @layer renderer-app @kind logic */
import type { BatchResult } from '../../../../../data/worktrees';
import { REASONS_SHOWN } from './model.constants';
import type { Notice } from './model.type';

const skippedText = (result: BatchResult): string => {
  const skipped = result.skipped ?? [];
  if (skipped.length === 0) return '';
  const why = [...new Set(skipped.map((item) => item.reason))].slice(0, REASONS_SHOWN).join('; ');
  return `, skipped ${skipped.length}${why ? ` (${why})` : ''}`;
};

const batchNotice = (result: BatchResult): Notice => {
  const started = result.started ?? 0;
  const folders = `${started} leftover ${started === 1 ? 'folder' : 'folders'}`;
  return {
    text: `Queued ${folders} for deletion${skippedText(result)}. Argus deletes them one after another in the background; each row goes when its folder is gone.`,
    backup: null,
    restore: null,
  };
};

export { batchNotice };
