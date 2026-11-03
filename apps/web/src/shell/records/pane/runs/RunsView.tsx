/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import { RunTerminal } from '../../../../screens/board/run-terminal';
import { DrawerSection } from '../../../../screens/board/tabs/kanban/sub-components/DrawerSection';
import { EmptyLine } from '../../../../screens/board/tabs/kanban/sub-components/EmptyLine';
import { BuildCommand } from './BuildCommand';
import { RunRows } from './RunRows';
import { COMMAND_TITLE, LOG_TAB, NO_RUNS_TEXT, RUNS_TITLE, TERMINAL_TITLE } from './runs.constants';
import type { RunsViewProps } from './runs.type';

const RunsView = (props: RunsViewProps) => {
  const { data, view, runs } = props;
  const { boardKey, open } = data;
  return (
    <Flex direction="column" gap="lg">
      <DrawerSection title={COMMAND_TITLE}>
        <BuildCommand view={view} boardKey={boardKey} runs={runs} />
      </DrawerSection>
      {view.playable && view.shown && (
        <DrawerSection title={TERMINAL_TITLE}>
          <RunTerminal key={view.shown.id} run={view.shown} boardKey={boardKey} onLog={(id) => open(id, LOG_TAB)} />
        </DrawerSection>
      )}
      <DrawerSection title={`${RUNS_TITLE} · ${view.runs.length}`}>
        {view.runs.length === 0 ? <EmptyLine>{NO_RUNS_TEXT}</EmptyLine> : <RunRows data={data} runs={view.runs} />}
      </DrawerSection>
    </Flex>
  );
};

export { RunsView };
