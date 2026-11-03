/* @layer renderer-app @kind logic */
import { kindOf } from '../../../../data/registry';
import type { RequestOutcome, RequestRecord } from '../../../../data/worktrees';
import { requestTitle } from '../../../../screens/board/tabs/worktrees/model/request-title';
import { shownCode } from '../../behavior/shown-code';
import type { PaneDoc } from '../pane.type';
import { WORKTREE_KIND } from './pane-model.constants';

const labelsOf = (results: readonly RequestOutcome[], ok: boolean): string =>
  results
    .filter((result) => result.ok === ok)
    .map((result) => (ok ? result.label : `${result.label}: ${result.text}`))
    .join('; ');

const worktreeCode = (id: string): string => {
  const kind = kindOf(WORKTREE_KIND);
  return kind ? shownCode(kind, id) : id;
};

const docOfRequest = (record: RequestRecord): PaneDoc & { id: string } => {
  const removed = record.results.filter((result) => result.ok).length;
  const refused = record.results.length - removed;
  const total = record.targets.length;
  return {
    id: record.id,
    title: requestTitle({ id: record.id, by: record.by, mode: record.mode, status: record.status, branch: record.branch, total, removed, refused }),
    by: record.by,
    at: record.at,
    mode: record.mode,
    status: record.status,
    reason: record.reason ?? '',
    note: record.note ?? '',
    requested: total,
    removed: labelsOf(record.results, true),
    refused: labelsOf(record.results, false),
    skipped: record.skipped?.count ?? 0,
    startedAt: record.startedAt ?? '',
    finishedAt: record.finishedAt ?? '',
    worktrees: record.targets.map((target) => worktreeCode(target.id)),
  };
};

const requestDocs = (records: readonly RequestRecord[]): (PaneDoc & { id: string })[] => records.map(docOfRequest);

export { requestDocs };
