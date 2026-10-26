/* @layer renderer-app @kind logic */
import type { TestNotice } from './model.type';

const conflictedTests = (notices: readonly TestNotice[]): Set<string> =>
  new Set(notices.filter((notice) => notice.kind === 'conflict' && notice.op.target?.id).map((notice) => String(notice.op.target?.id)));

export { conflictedTests };
