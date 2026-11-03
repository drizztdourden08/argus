/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { useBuildsData } from '../../../../screens/board/tabs/builds';
import { EmptyLine } from '../../../../screens/board/tabs/kanban/sub-components/EmptyLine';
import type { RecordBodyProps } from '../pane.type';
import { LOADING_TEXT, NO_BUILD_TEXT } from '../runs/runs.constants';
import { RunsView } from '../runs/RunsView';

const RunsTab = (props: RecordBodyProps) => {
  const { data } = props;
  const builds = useBuildsData(data.boardKey);
  if (!builds.ready) return <Text tone="muted">{LOADING_TEXT}</Text>;
  const view = [...builds.views.live, ...builds.views.retired].find((candidate) => candidate.launcher.name === data.target.id);
  if (!view) return <EmptyLine>{NO_BUILD_TEXT}</EmptyLine>;
  return <RunsView data={data} view={view} runs={builds.runs} />;
};

export { RunsTab };
