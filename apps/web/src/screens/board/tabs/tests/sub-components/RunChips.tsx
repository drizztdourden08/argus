/* @layer renderer-app @kind component */
import { Tag } from '@drizztdourden08/tessera';
import { useRecordPane } from '../../../../../shell';
import { planCode } from '../model';
import { RUN_TAGS, RUN_TIPS } from '../tests.constants';
import type { RunChipsProps } from '../tests-head-props.type';
import { BuildChip } from './BuildChip';

const RunChips = (props: RunChipsProps) => {
  const { view } = props;
  const { stage, handed, plan, build, launch, item } = view;
  const pane = useRecordPane();
  const shownPlan = plan ? planCode(plan.id) : '';
  return (
    <>
      <Tag {...(RUN_TAGS[stage] ?? RUN_TAGS.draft)} fill="tint" title={RUN_TIPS[stage]} data-rstage={stage}>{stage}</Tag>
      {plan && (
        <Tag icon="scroll-text" title={plan.title} aria-label={`Open plan ${shownPlan}: ${plan.title}`} onClick={() => pane.open(shownPlan)}>{shownPlan}</Tag>
      )}
      {build && <BuildChip build={build} gone={launch.gone} />}
      {handed && <Tag color="neutral" fill="solid" icon="hand" title={`${item.id} is the run handed to you`}>handed to you</Tag>}
    </>
  );
};

export { RunChips };
