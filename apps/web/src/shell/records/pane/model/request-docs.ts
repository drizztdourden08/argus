/* @layer renderer-app @kind logic */
import { kindOf } from '../../../../data/registry';
import { requestTitle } from '../../../../data/worktrees';
import type { RequestOutcome, RequestRecord, WorktreeRequest } from '../../../../data/worktrees';
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

const briefOf = (record: RequestRecord): WorktreeRequest => {
  const removed = record.results.filter((result) => result.ok).length;
  const { id, by, mode, status, branch } = record;
  return { id, by, mode, status, branch, total: record.targets.length, removed, refused: record.results.length - removed };
};

const docOfRequest = (record: RequestRecord): PaneDoc & { id: string } => {
  const brief = briefOf(record);
  return {
    id: record.id,
    title: requestTitle(brief),
    by: record.by,
    at: record.at,
    mode: record.mode,
    status: record.status,
    reason: record.reason ?? '',
    note: record.note ?? '',
    requested: brief.total,
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
