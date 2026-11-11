/* @layer renderer-app @kind logic */
import { useMemo, useState } from 'react';
import type { ReviewDoc } from '../../../../../data/types';
import { accountLabels, applyCount, folderMembers, pendingKeys, titleMap, visibleRows } from '../model';
import { NO_ROWS } from '../review.constants';
import type { ReviewDeck, ReviewFilter } from '../review.type';
import { useCollapsed } from './useCollapsed';
import { useDecisions } from './useDecisions';
import { useReviewApply } from './useReviewApply';
import { useReviewDialog } from './useReviewDialog';
import { useRowActions } from './useRowActions';
import { useSelection } from './useSelection';
import { useStagedEdits } from './useStagedEdits';

const useReviewDeck = (data: ReviewDoc | null, onApplied: () => void): ReviewDeck => {
  const rows = data?.rows ?? NO_ROWS;
  const accounts = data?.accounts ?? NO_ROWS;
  const [filter, setFilter] = useState<ReviewFilter>('all');
  const choices = useDecisions();
  const staged = useStagedEdits();
  const collapse = useCollapsed();
  const picks = useSelection();
  const dialog = useReviewDialog();
  const submit = useReviewApply(onApplied);
  const onAct = useRowActions(staged.add, dialog);
  const edits = useMemo(() => staged.items.map((item) => item.edit), [staged.items]);
  const shown = useMemo(() => visibleRows(rows, filter, collapse.collapsed), [rows, filter, collapse.collapsed]);
  const members = useMemo(() => new Map(accounts.map((account) => [account.key, folderMembers(rows, account.key)])), [accounts, rows]);
  const names = useMemo(() => ({ titles: titleMap(rows), accounts: accountLabels(accounts) }), [rows, accounts]);
  const env = {
    decisions: choices.decisions,
    collapsed: collapse.collapsed,
    selection: picks.selection,
    pending: pendingKeys(edits),
    members,
    now: Date.now(),
    onToggle: collapse.toggle,
    onSelect: picks.set,
    onAct,
    onChoose: (rowId: string, choice: string, fallback: string) => choices.choose(rowId, choice, fallback),
  };
  return {
    rows, accounts, shown, filter, setFilter, choices, staged, edits, picks, dialog, submit, names, env,
    changed: Object.keys(choices.decisions).length,
    count: applyCount(rows, choices.decisions, edits.length),
  };
};

export { useReviewDeck };
