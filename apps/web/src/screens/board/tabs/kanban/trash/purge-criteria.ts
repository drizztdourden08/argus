/* @layer renderer-app @kind logic */
import type { PurgeCriteria, PurgeDraft } from './trash.type';

const purgeCriteria = (draft: PurgeDraft): PurgeCriteria => {
  switch (draft.scope) {
    case 'thread':
      return { threads: draft.thread ? [draft.thread] : [] };
    case 'keep':
      return { keep: [...draft.keep] };
    case 'status':
      return { status: [...draft.status] };
    case 'idle':
      return { idleDays: draft.idle };
    case 'trash':
      return { trash: true };
  }
};

export { purgeCriteria };
