/* @layer renderer-app @kind component */
import { Callout, Card, EmptyState, Flex, SectionHeader, Text } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { useBuildsData } from './behavior/useBuildsData';
import { useRetiredShown } from './behavior/useRetiredShown';
import { EMPTY_HINT, EMPTY_TEXT, NO_MASTER_TEXT } from './builds.constants';
import type { BuildsTabProps } from './builds-props.type';
import { countsText } from './model';
import { RunLogDrawer } from '../../run-terminal';
import { BuildsLead } from './sub-components/BuildsLead';
import { LauncherList } from './sub-components/LauncherList';
import { RetiredSection } from './sub-components/RetiredSection';

const BuildsTab = (props: BuildsTabProps) => {
  const { boardKey } = props;
  const data = useBuildsData(boardKey);
  const flag = useRetiredShown();
  const [logId, setLogId] = useState<string | null>(null);
  const { views, runs } = data;
  if (!data.ready) return <Text tone="muted">Loading the builds.</Text>;
  const counts = countsText(views);
  return (
    <>
      <Card>
        <Flex direction="column" gap="sm">
          <SectionHeader size="lg" wrap level={2} title="Builds" subtitle={<BuildsLead counts={counts} />} />
          {!views.hasMaster && <Callout tone="danger">{NO_MASTER_TEXT}</Callout>}
          {views.live.length > 0
            ? <LauncherList views={views.live} label="Launchers" boardKey={boardKey} runs={runs} onLog={setLogId} />
            : <EmptyState message={EMPTY_TEXT} hint={EMPTY_HINT} />}
          <RetiredSection views={views.retired} flag={flag} boardKey={boardKey} runs={runs} onLog={setLogId} />
        </Flex>
      </Card>
      <RunLogDrawer run={runs.find((run) => run.id === logId) ?? null} boardKey={boardKey} onClose={() => setLogId(null)} />
    </>
  );
};

export { BuildsTab };
