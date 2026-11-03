/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { LauncherList } from '../../../../screens/board/tabs/builds';
import { DrawerSection } from '../../../../screens/board/tabs/kanban/sub-components/DrawerSection';
import { EmptyLine } from '../../../../screens/board/tabs/kanban/sub-components/EmptyLine';
import type { PaneBodyProps } from '../pane.type';
import { useServed } from '../served/behavior/useServed';
import { BUILDS_TITLE, NO_BUILDS_TEXT } from '../served/served.constants';
import { ServedWorktrees } from '../served/ServedWorktrees';

const ServedTab = (props: PaneBodyProps) => {
  const { boardKey, item } = props.data;
  const served = useServed(boardKey, item.id);
  if (!served.ready) return <Text tone="muted">Loading the builds.</Text>;
  return (
    <Flex direction="column" gap="lg">
      <DrawerSection title={`${BUILDS_TITLE} · ${served.builds.length}`}>
        {served.builds.length === 0
          ? <EmptyLine>{NO_BUILDS_TEXT}</EmptyLine>
          : <LauncherList views={served.builds} label={BUILDS_TITLE} boardKey={boardKey} runs={served.runs} />}
      </DrawerSection>
      <ServedWorktrees rows={served.worktrees} />
    </Flex>
  );
};

export { ServedTab };
