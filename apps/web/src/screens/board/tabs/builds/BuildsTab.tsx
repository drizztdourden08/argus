/* @layer renderer-app @kind component */
import { Callout, EmptyState, Text } from '@drizztdourden08/tessera';
import { useBuildsData } from './behavior/useBuildsData';
import { useRetiredShown } from './behavior/useRetiredShown';
import { EMPTY_HINT, EMPTY_TEXT, NO_MASTER_TEXT } from './builds.constants';
import type { BuildsTabProps } from './builds-props.type';
import { countsText } from './model';
import { Panel } from '../../../../shell';
import { BuildsLead } from './sub-components/BuildsLead';
import { BuildSections } from './sub-components/BuildSections';
import { RetiredSection } from './sub-components/RetiredSection';

const BuildsTab = (props: BuildsTabProps) => {
  const { boardKey } = props;
  const data = useBuildsData(boardKey);
  const flag = useRetiredShown();
  const { views, groups, runs, threads } = data;
  if (!data.ready) return <Text tone="muted">Loading the builds.</Text>;
  return (
    <Panel title="Builds" note={<BuildsLead counts={countsText(views)} />}>
      {!views.hasMaster && <Callout tone="danger">{NO_MASTER_TEXT}</Callout>}
      {views.live.length > 0
        ? <BuildSections groups={groups} boardKey={boardKey} runs={runs} threads={threads} />
        : <EmptyState message={EMPTY_TEXT} hint={EMPTY_HINT} />}
      <RetiredSection views={groups.closed} flag={flag} boardKey={boardKey} runs={runs} />
    </Panel>
  );
};

export { BuildsTab };
