/* @layer renderer-app @kind component */
import { Button, Flex, SegmentedControl, Text } from '@drizztdourden08/tessera';
import { TabFilters } from '../../../BoardTabsRow';
import { aboutSize, daysText } from '../model';
import type { WorktreeFilter } from '../model';
import { EXPLAINER_TEXT, FILTER_LABEL } from '../worktrees.constants';
import type { WorktreeToolbarProps } from '../worktrees-props.type';
import './WorktreeToolbar.css';

const WorktreeToolbar = (props: WorktreeToolbarProps) => {
  const { filter, onFilter, counts, meta, batch } = props;
  const about = <Text variant="caption" tone="dim" className="worktree-toolbar__about">{EXPLAINER_TEXT}</Text>;
  if (counts.all === 0) return about;
  const old = meta?.leftoverOld;
  const options: { value: WorktreeFilter; label: string }[] = [
    { value: 'all', label: `All (${counts.all})` },
    { value: 'need', label: `Need a decision (${counts.need})` },
  ];
  if (counts.left > 0) options.push({ value: 'left', label: `Leftovers (${counts.left})` });
  return (
    <Flex gap="md" align="center" wrap>
      {about}
      <TabFilters tab="worktrees" count={filter === 'all' ? 0 : 1}>
        <SegmentedControl<WorktreeFilter> selectedTone="neutral" aria-label={FILTER_LABEL} size="sm" value={filter} onChange={onFilter} options={options} />
      </TabFilters>
      {old && old.count > 0 && (
        <Button size="sm" variant="danger" aria-expanded={batch.state !== null} disabled={batch.busy} onClick={batch.toggle}>
          {`Delete all leftovers older than ${daysText(old.days)} (${old.count}, ${aboutSize(old.sizeMB)})`}
        </Button>
      )}
    </Flex>
  );
};

export { WorktreeToolbar };
