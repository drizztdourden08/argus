/* @layer renderer-app @kind component */
import './BundleCard.css';
import { Card, Disclosure } from '@drizztdourden08/tessera';
import { bundleState } from '../../../shared';
import { NO_BUNDLE } from '../model';
import type { BundleCardProps } from '../tests-props.type';
import { BundleHead } from './BundleHead';
import { BundleName } from './BundleName';
import { LaunchBox } from './LaunchBox';
import { TestSections } from './TestSections';

const BundleCard = (props: BundleCardProps) => {
  const { view, runs, open, onToggle } = props;
  const { group } = view;
  const state = bundleState(group.build);
  const emptyText = group.tag === NO_BUNDLE ? 'No test without a bundle.' : 'No test in this bundle yet.';
  return (
    <Card flush className="bundle" edgeTone={group.handed ? 'neutral' : undefined} data-bundle={group.tag}>
      <Disclosure
        open={open}
        onOpenChange={onToggle}
        summary={<BundleName tag={group.tag} title={group.build?.title} />}
        end={<BundleHead group={group} state={state} />}
      >
        <LaunchBox view={view} runs={runs} state={state} />
        <TestSections tests={group.tests} emptyText={emptyText} />
      </Disclosure>
    </Card>
  );
};

export { BundleCard };
