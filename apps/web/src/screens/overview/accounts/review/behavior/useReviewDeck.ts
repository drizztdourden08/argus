/* @layer renderer-app @kind logic */
import { useMemo, useState } from 'react';
import type { ReviewDoc } from '../../../../../data/types';
import { accountLabels, applyCount, folderNames, pendingKeys, scopeMap, titleMap, visibleRows } from '../model';
import type { TreeEnv } from '../review-tree-props.type';
import { NO_ROWS } from '../review.constants';
import type { ReviewDeck, ReviewFilter } from '../review.type';
import { useCollapsed } from './useCollapsed';
import { useDecisions } from './useDecisions';
import { useReviewApply } from './useReviewApply';
import { useReviewDialog } from './useReviewDialog';
import { useRowActions } from './useRowActions';
import { useStagedEdits } from './useStagedEdits';

const useReviewDeck = (data: ReviewDoc | null, onApplied: () => void): ReviewDeck => {
  const rows = data?.rows ?? NO_ROWS;
  const accounts = data?.accounts ?? NO_ROWS;
  const [filter, setFilter] = useState<ReviewFilter>('all');
  const [hovered, setHovered] = useState<string | null>(null);
  const choices = useDecisions();
  const staged = useStagedEdits();
  const collapse = useCollapsed();
  const dialog = useReviewDialog();
  const submit = useReviewApply(onApplied);
  const onAct = useRowActions(staged.add, dialog);
  const edits = useMemo(() => staged.items.map((item) => item.edit), [staged.items]);
  const shown = useMemo(() => visibleRows(rows, filter, collapse.collapsed), [rows, filter, collapse.collapsed]);
  const scopes = useMemo(() => scopeMap(rows), [rows]);
  const folders = useMemo(() => new Map(accounts.map((account) => [account.key, folderNames(rows, account.key)])), [accounts, rows]);
  const names = useMemo(() => ({ titles: titleMap(rows), accounts: accountLabels(accounts) }), [rows, accounts]);
  const now = useMemo(() => Date.now(), []);
  const pending = useMemo(() => pendingKeys(edits), [edits]);
  const pair = { left: accounts[0]?.key ?? '', right: accounts[1]?.key ?? '' };
  const env: TreeEnv = useMemo(
    () => ({ decisions: choices.decisions, collapsed: collapse.collapsed, pending, folders, scopes, pair: { left: pair.left, right: pair.right }, now, onHover: setHovered, onToggle: collapse.toggle, onAct, onStage: staged.add, onChoose: choices.choose }),
    [choices.decisions, choices.choose, collapse.collapsed, collapse.toggle, pending, folders, scopes, pair.left, pair.right, now, onAct, staged.add],
  );
  return {
    rows, accounts, shown, filter, setFilter, choices, staged, edits, dialog, submit, names, env, hovered, setHovered,
    expandAll: () => collapse.setAll(new Set()),
    collapseAll: () => collapse.setAll(new Set([...scopes].filter(([, kids]) => kids.length > 0).map(([id]) => id))),
    changed: Object.keys(choices.decisions).length,
    count: applyCount(rows, choices.decisions, edits.length),
  };
};

export { useReviewDeck };
