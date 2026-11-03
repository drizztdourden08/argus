/* @layer renderer-app @kind logic */
import { shortCommit } from '../../../../format';
import type { CheckRun } from '../../../../data/remote';
import type { PaneDoc } from '../pane.type';

const checkDocs = (runs: readonly CheckRun[]): (PaneDoc & { id: string })[] =>
  runs.map((run) => ({
    id: run.id,
    name: `${run.trigger} on ${shortCommit(run.commit)}`,
    status: run.status,
    branch: run.branch,
    commit: run.commit,
    trigger: run.trigger,
    startedAt: run.startedAt ?? '',
    finishedAt: run.finishedAt ?? '',
    summary: run.summary,
    change: run.pr ?? '',
  }));

export { checkDocs };
