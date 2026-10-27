/* @layer renderer-app @kind component */
import './BundleCard.css';
import { Box } from '@drizztdourden08/tessera';
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
    <Box className={group.handed ? 'bundle bundle--handed' : 'bundle'} data-bundle={group.tag}>
      <BundleHead group={group} state={state} open={open} onToggle={onToggle} />
      <Box className="bundle__body" hidden={!open}>
        <LaunchBox view={view} runs={runs} state={state} />
        <TestSections tests={group.tests} emptyText={emptyText} />
      </Box>
    </Box>
  );
};

export { BundleCard };
