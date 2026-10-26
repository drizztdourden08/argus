/* @layer renderer-app @kind component */
import { Card, Disclosure, Flex } from '@drizztdourden08/tessera';
import { bundleState, NO_BUNDLE } from '../model';
import type { BundleCardProps } from '../tests-props.type';
import { BundleHead } from './BundleHead';
import { LaunchBox } from './LaunchBox';
import { TestSections } from './TestSections';

const BundleCard = (props: BundleCardProps) => {
  const { view, boardKey, runs, open, onToggle, onOpenTest } = props;
  const { group } = view;
  const state = bundleState(group.build);
  const emptyText = group.tag === NO_BUNDLE ? 'No test without a bundle.' : 'No test in this bundle yet.';
  return (
    <Card data-bundle={group.tag}>
      <Disclosure open={open} onOpenChange={onToggle} summary={<BundleHead group={group} state={state} />}>
        <Flex direction="column" gap="md">
          <LaunchBox view={view} boardKey={boardKey} runs={runs} state={state} />
          <TestSections tests={group.tests} emptyText={emptyText} boardKey={boardKey} onOpenTest={onOpenTest} />
        </Flex>
      </Disclosure>
    </Card>
  );
};

export { BundleCard };
