/* @layer renderer-app @kind component */
import { Button, Flex, SegmentedControl } from '@drizztdourden08/tessera';
import { aboutSize, daysText } from '../model';
import type { WorktreeFilter } from '../model';
import { FILTER_LABEL } from '../worktrees.constants';
import type { WorktreeToolbarProps } from '../worktrees-props.type';

const WorktreeToolbar = (props: WorktreeToolbarProps) => {
  const { filter, onFilter, counts, meta, batch } = props;
  if (counts.all === 0) return null;
  const old = meta?.leftoverOld;
  const options: { value: WorktreeFilter; label: string }[] = [
    { value: 'all', label: `All (${counts.all})` },
    { value: 'need', label: `Need a decision (${counts.need})` },
  ];
  if (counts.left > 0) options.push({ value: 'left', label: `Leftovers (${counts.left})` });
  return (
    <Flex gap="sm" align="center" wrap>
      <SegmentedControl<WorktreeFilter> aria-label={FILTER_LABEL} size="sm" value={filter} onChange={onFilter} options={options} />
      {old && old.count > 0 && (
        <Button size="sm" variant="danger" aria-expanded={batch.state !== null} disabled={batch.busy} onClick={batch.toggle}>
          {`Delete all leftovers older than ${daysText(old.days)} (${old.count}, ${aboutSize(old.sizeMB)})`}
        </Button>
      )}
    </Flex>
  );
};

export { WorktreeToolbar };
