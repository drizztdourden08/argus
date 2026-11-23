/* @layer renderer-app @kind component */
import { Button } from '@drizztdourden08/tessera';
import { TabFilterBar, TabGroupSort, ToolToggle, useTabTools } from '../../../BoardTabsRow';
import { LayoutToggle } from '../../../shared';
import { aboutSize, daysText, rowFields } from '../model';
import { COMPACT_LABEL, GROUP_CHOICES, LAYOUT_CHOICES, NEED_LABEL, ORDER_CHOICES, WORKTREES_TAB } from '../worktrees.constants';
import type { WorktreeToolbarProps } from '../worktrees-props.type';

const WorktreeToolbar = (props: WorktreeToolbarProps) => {
  const { boardKey, rows, prefs, counts, meta, batch } = props;
  const { view } = prefs;
  const old = meta?.leftoverOld;
  const sort = { options: ORDER_CHOICES, order: view.order, direction: view.direction, onChange: view.setSort };
  const clean = old && old.count > 0 && (
    <Button size="sm" variant="danger" aria-expanded={batch.state !== null} disabled={batch.busy} onClick={batch.toggle}>
      {`Delete all leftovers older than ${daysText(old.days)} (${old.count}, ${aboutSize(old.sizeMB)})`}
    </Button>
  );
  const compact = { id: 'compact', priority: 2, node: <ToolToggle label={COMPACT_LABEL} pressed={prefs.compact} onToggle={() => prefs.setCompact(!prefs.compact)} /> };
  const items = counts.all === 0 ? [] : [
    { id: 'need', priority: 4, node: <ToolToggle label={NEED_LABEL} count={counts.need} pressed={prefs.needOnly} onToggle={() => prefs.setNeedOnly(!prefs.needOnly)} /> },
    { id: 'group-sort', priority: 5, node: <TabGroupSort group={{ options: GROUP_CHOICES, value: view.group, empty: 'none', onChange: view.setGroup }} sort={sort} /> },
    { id: 'layout', priority: 3, node: <LayoutToggle value={prefs.layout} options={LAYOUT_CHOICES} onChange={prefs.setLayout} label="Worktree view" /> },
    ...(prefs.layout === 'cards' ? [compact] : []),
    ...(clean ? [{ id: 'delete-leftovers', priority: 1, node: clean }] : []),
  ];
  useTabTools(WORKTREES_TAB, items);
  if (counts.all === 0) return null;
  return (
    <TabFilterBar
      tab={WORKTREES_TAB}
      boardKey={boardKey}
      fields={rowFields(rows, meta)}
      filters={view.filters}
      patchFilters={view.patchFilters}
      placeholder="Search path or branch"
    />
  );
};

export { WorktreeToolbar };
