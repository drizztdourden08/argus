/* @layer renderer-app @kind component */
import { Button, Inline, SegmentedControl, Stack, Text } from '@drizztdourden08/tessera';
import { FILTER_OPTIONS } from '../review.constants';
import type { ReviewFilter } from '../review.type';
import type { ReviewToolbarProps } from '../review-props.type';
import { ExpandBar } from './ExpandBar';
import { ReviewChips } from './ReviewChips';
import { ScanStatus } from './ScanStatus';

const ReviewToolbar = (props: ReviewToolbarProps) => {
  const { accounts, counts, edits, filter, loading, applyCount, onFilter, onBack, onRescan, onExpandAll, onCollapseAll, onCleanup, onApply } = props;
  return (
    <Stack gap="sm">
      <Inline gap="md" align="center" wrap>
        <Button size="sm" variant="ghost" onClick={onBack}>Back</Button>
        <Text size="md" weight={600}>Review</Text>
        <ReviewChips accounts={accounts} counts={counts} edits={edits} />
        <SegmentedControl<ReviewFilter> size="sm" label="Show rows" value={filter} options={[...FILTER_OPTIONS]} onChange={onFilter} />
        <ExpandBar onExpandAll={onExpandAll} onCollapseAll={onCollapseAll} />
        <Button size="sm" variant="ghost" onClick={loading ? undefined : onRescan}>Rescan</Button>
        <Button size="sm" variant="ghost" onClick={onCleanup}>Cleanup...</Button>
        <Button size="sm" disabled={applyCount === 0} onClick={onApply}>{`Apply ${applyCount} changes`}</Button>
      </Inline>
      <ScanStatus loading={loading} />
    </Stack>
  );
};

export { ReviewToolbar };
