/* @layer renderer-app @kind component */
import { Callout, EmptyState, Flex, Text } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { useBuildsData } from './behavior/useBuildsData';
import { useRetiredShown } from './behavior/useRetiredShown';
import { EMPTY_HINT, EMPTY_TEXT, NO_MASTER_TEXT } from './builds.constants';
import type { BuildsTabProps } from './builds-props.type';
import { BuildsHeader } from './sub-components/BuildsHeader';
import { LauncherList } from './sub-components/LauncherList';
import { RetiredSection } from './sub-components/RetiredSection';
import { RunLogDrawer } from './sub-components/RunLogDrawer';
import { RunsSection } from './sub-components/RunsSection';

const BuildsTab = (props: BuildsTabProps) => {
  const { boardKey } = props;
  const data = useBuildsData(boardKey);
  const flag = useRetiredShown();
  const [logId, setLogId] = useState<string | null>(null);
  const { views, groups, runs } = data;
  if (!data.ready) return <Text tone="muted">Loading the builds.</Text>;
  return (
    <Flex direction="column" gap="md">
      <BuildsHeader views={views} />
      {!views.hasMaster && <Callout tone="danger">{NO_MASTER_TEXT}</Callout>}
      {views.live.length > 0
        ? <LauncherList views={views.live} label="Launchers" boardKey={boardKey} runs={runs} onLog={setLogId} />
        : <EmptyState message={EMPTY_TEXT} hint={EMPTY_HINT} />}
      <RetiredSection views={views.retired} flag={flag} boardKey={boardKey} runs={runs} onLog={setLogId} />
      <RunsSection groups={groups} boardKey={boardKey} onLog={setLogId} />
      <RunLogDrawer run={runs.find((run) => run.id === logId) ?? null} boardKey={boardKey} onClose={() => setLogId(null)} />
    </Flex>
  );
};

export { BuildsTab };
