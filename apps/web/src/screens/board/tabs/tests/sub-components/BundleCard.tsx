/* @layer renderer-app @kind component */
import { Card, Disclosure, Flex } from '@drizztdourden08/tessera';
import { bundleState, NO_BUNDLE } from '../model';
import type { BundleCardProps } from '../tests-props.type';
import { BundleHead } from './BundleHead';
import { LaunchBox } from './LaunchBox';
import { TestSections } from './TestSections';

const BundleCard = (props: BundleCardProps) => {
  const { view, runs, open, onToggle } = props;
  const { group } = view;
  const state = bundleState(group.build);
  const emptyText = group.tag === NO_BUNDLE ? 'No test without a bundle.' : 'No test in this bundle yet.';
  return (
    <Card data-bundle={group.tag}>
      <Disclosure open={open} onOpenChange={onToggle} summary={<BundleHead group={group} state={state} />}>
        <Flex direction="column" gap="md">
          <LaunchBox view={view} runs={runs} state={state} />
          <TestSections tests={group.tests} emptyText={emptyText} />
        </Flex>
      </Disclosure>
    </Card>
  );
};

export { BundleCard };
