/* @layer renderer-app @kind component */
import { DrawerSection } from '../../sub-components/DrawerSection';
import { EmptyLine } from '../../sub-components/EmptyLine';
import { caseIds } from '../model/case-ids';
import { useRunOps } from './behavior/useRunOps';
import { runsOf } from './model/runs-of';
import { shownRun } from './model/shown-run';
import { RunStage } from './RunStage';
import { RunStart } from './RunStart';
import { RunSummary } from './RunSummary';
import type { RunPanelProps } from './run.type';

const RunPanel = (props: RunPanelProps) => {
  const { data } = props;
  const { item, boardKey } = data;
  const ids = caseIds(item);
  const ops = useRunOps(boardKey, item.id);
  const run = shownRun(runsOf(item));
  const bundle = typeof item.fields?.bundle === 'string' ? item.fields.bundle : null;
  const open = run && run.endedAt === null ? run : null;
  const ended = run && run.endedAt !== null ? run : null;
  return (
    <DrawerSection title="Run">
      {ids.length === 0 && <EmptyLine>Add cases to this plan, then start a run.</EmptyLine>}
      {ids.length > 0 && !run && <RunStart bundle={bundle} ops={ops} again={false} />}
      {ids.length > 0 && open && <RunStage key={open.n} data={data} run={open} ids={ids} ops={ops} />}
      {ids.length > 0 && ended && <RunSummary data={data} run={ended} ids={ids} ops={ops} />}
    </DrawerSection>
  );
};

export { RunPanel };
