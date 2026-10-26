/* @layer renderer-app @kind component */
import { EmptyState, Flex } from '@drizztdourden08/tessera';
import { useTestsShared } from '../behavior/useTestsShared';
import { NO_MATCH_TEXT, NO_TESTS_TEXT } from '../tests.constants';
import type { BundleListProps } from '../tests-props.type';
import { BundleCard } from './BundleCard';

const BundleList = (props: BundleListProps) => {
  const { bundles, runs, filtered } = props;
  const { folds } = useTestsShared();
  if (bundles.length === 0) return <EmptyState message={filtered ? NO_MATCH_TEXT : NO_TESTS_TEXT} />;
  return (
    <Flex direction="column" gap="md">
      {bundles.map((view) => (
        <BundleCard
          key={view.group.tag}
          view={view}
          runs={runs}
          open={folds.isOpen(view.group.tag, view.group.open)}
          onToggle={(open) => folds.set(view.group.tag, open)}
        />
      ))}
    </Flex>
  );
};

export { BundleList };
