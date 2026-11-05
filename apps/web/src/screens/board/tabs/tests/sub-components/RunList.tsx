/* @layer renderer-app @kind component */
import { EmptyState, Flex } from '@drizztdourden08/tessera';
import { useTestsShared } from '../behavior/useTestsShared';
import { NO_MATCH_TEXT, NO_RUNS_TEXT } from '../tests.constants';
import type { RunListProps } from '../tests-props.type';
import { RunCard } from './RunCard';

const RunList = (props: RunListProps) => {
  const { views, runs, filtered } = props;
  const { folds } = useTestsShared();
  if (views.length === 0) return <EmptyState message={filtered ? NO_MATCH_TEXT : NO_RUNS_TEXT} />;
  return (
    <Flex direction="column" gap="md">
      {views.map((view) => (
        <RunCard
          key={view.item.id}
          view={view}
          runs={runs}
          open={folds.isOpen(view.item.id, view.open)}
          onToggle={(open) => folds.set(view.item.id, open)}
        />
      ))}
    </Flex>
  );
};

export { RunList };
