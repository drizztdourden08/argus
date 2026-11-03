/* @layer renderer-app @kind component */
import { DrawerSection } from '../../sub-components/DrawerSection';
import { EmptyLine } from '../../sub-components/EmptyLine';
import { caseIds } from '../model/case-ids';
import { useRunOps } from './behavior/useRunOps';
import { bundleOf } from './model/bundle-of';
import { runsOf } from './model/runs-of';
import { shownRun } from './model/shown-run';
import { RunBody } from './RunBody';
import type { RunPanelProps } from './run.type';

const RunPanel = (props: RunPanelProps) => {
  const { data } = props;
  const { item, boardKey } = data;
  const ids = caseIds(item);
  const ops = useRunOps(boardKey, item.id);
  const run = shownRun(runsOf(item));
  return (
    <DrawerSection title="Run">
      {ids.length === 0 ? (
        <EmptyLine>Add cases to this plan, then start a run.</EmptyLine>
      ) : (
        <RunBody data={data} run={run} ids={ids} ops={ops} bundle={bundleOf(item)} />
      )}
    </DrawerSection>
  );
};

export { RunPanel };
