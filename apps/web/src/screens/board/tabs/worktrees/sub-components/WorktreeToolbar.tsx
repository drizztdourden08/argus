/* @layer renderer-app @kind component */
import { Button, Flex, SegmentedControl, Text } from '@drizztdourden08/tessera';
import { TabFilterBar, TabGroupSort, useTabTools } from '../../../BoardTabsRow';
import { aboutSize, daysText, rowFields } from '../model';
import type { WorktreeFilter } from '../model';
import { EXPLAINER_TEXT, FILTER_LABEL, GROUP_CHOICES, ORDER_CHOICES } from '../worktrees.constants';
import type { WorktreeToolbarProps } from '../worktrees-props.type';
import './WorktreeToolbar.css';

const WorktreeToolbar = (props: WorktreeToolbarProps) => {
  const { boardKey, rows, prefs, counts, meta, batch } = props;
  const { view } = prefs;
  const old = meta?.leftoverOld;
  const options: { value: WorktreeFilter; label: string }[] = [
    { value: 'all', label: `All (${counts.all})` },
    { value: 'need', label: `Need a decision (${counts.need})` },
  ];
  if (counts.left > 0) options.push({ value: 'left', label: `Leftovers (${counts.left})` });
  const show = (
    <Flex inline gap="xs" align="center">
      <Text variant="caption" tone="muted">{FILTER_LABEL}</Text>
      <SegmentedControl<WorktreeFilter> selectedTone="neutral" aria-label={FILTER_LABEL} size="sm" value={prefs.filter} onChange={prefs.setFilter} options={options} />
    </Flex>
  );
  const sort = {
    options: ORDER_CHOICES,
    order: view.order,
    direction: view.direction,
    onChange: view.setSort,
  };
  const clean = old && old.count > 0 && (
    <Button size="sm" variant="danger" aria-expanded={batch.state !== null} disabled={batch.busy} onClick={batch.toggle}>
      {`Delete all leftovers older than ${daysText(old.days)} (${old.count}, ${aboutSize(old.sizeMB)})`}
    </Button>
  );
  const items = counts.all === 0 ? [] : [
    { id: 'show', priority: 4, node: show },
    { id: 'group-sort', priority: 5, node: <TabGroupSort group={{ options: GROUP_CHOICES, value: view.group, empty: 'none', onChange: view.setGroup }} sort={sort} /> },
    ...(clean ? [{ id: 'delete-leftovers', priority: 1, node: clean }] : []),
  ];
  useTabTools('worktrees', items);
  return (
    <>
      <Text variant="caption" tone="dim" className="worktree-toolbar__about">{EXPLAINER_TEXT}</Text>
      {counts.all > 0 && (
        <TabFilterBar
          tab="worktrees"
          boardKey={boardKey}
          fields={rowFields(rows, meta)}
          filters={view.filters}
          patchFilters={view.patchFilters}
          placeholder="Search path or branch"
        />
      )}
    </>
  );
};

export { WorktreeToolbar };
