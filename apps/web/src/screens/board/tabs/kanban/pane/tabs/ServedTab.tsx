/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { LauncherList } from '../../../builds';
import { RunLogDrawer } from '../../../../run-terminal';
import { DrawerSection } from '../../sub-components/DrawerSection';
import { EmptyLine } from '../../sub-components/EmptyLine';
import type { PaneBodyProps } from '../pane.type';
import { useServed } from '../served/behavior/useServed';
import { BUILDS_TITLE, NO_BUILDS_TEXT } from '../served/served.constants';
import { ServedWorktrees } from '../served/ServedWorktrees';

const ServedTab = (props: PaneBodyProps) => {
  const { boardKey, item } = props.data;
  const served = useServed(boardKey, item.id);
  const [logId, setLogId] = useState<string | null>(null);
  if (!served.ready) return <Text tone="muted">Loading the builds.</Text>;
  return (
    <Flex direction="column" gap="lg">
      <DrawerSection title={`${BUILDS_TITLE} · ${served.builds.length}`}>
        {served.builds.length === 0
          ? <EmptyLine>{NO_BUILDS_TEXT}</EmptyLine>
          : <LauncherList views={served.builds} label={BUILDS_TITLE} boardKey={boardKey} runs={served.runs} onLog={setLogId} />}
      </DrawerSection>
      <ServedWorktrees rows={served.worktrees} />
      <RunLogDrawer run={served.runs.find((run) => run.id === logId) ?? null} boardKey={boardKey} onClose={() => setLogId(null)} />
    </Flex>
  );
};

export { ServedTab };
