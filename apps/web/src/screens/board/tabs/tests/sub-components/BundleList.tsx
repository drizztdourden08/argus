/* @layer renderer-app @kind component */
import { EmptyState, Flex } from '@drizztdourden08/tessera';
import type { BundleListProps } from '../tests-props.type';
import { BundleCard } from './BundleCard';

const BundleList = (props: BundleListProps) => {
  const { bundles, boardKey, runs, folds, onOpenTest } = props;
  if (bundles.length === 0) return <EmptyState message="No test yet. Threads document every test here with coord test." />;
  return (
    <Flex direction="column" gap="md">
      {bundles.map((view) => (
        <BundleCard
          key={view.group.tag}
          view={view}
          boardKey={boardKey}
          runs={runs}
          open={folds.isOpen(view.group.tag, view.group.open)}
          onToggle={(open) => folds.set(view.group.tag, open)}
          onOpenTest={onOpenTest}
        />
      ))}
    </Flex>
  );
};

export { BundleList };
