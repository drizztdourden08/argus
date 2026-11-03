/* @layer renderer-app @kind component */
import { Card } from '@drizztdourden08/tessera';
import { RecordCard, useRecordPane } from '../../../../../shell';
import { useBuildHead } from '../behavior/useBuildHead';
import type { LauncherCardProps } from '../builds-props.type';
import { buildTitle } from '../model';
import { BuildFoot } from './BuildFoot';
import { BuildSignals } from './BuildSignals';

const LauncherCard = (props: LauncherCardProps) => {
  const { view, boardKey, runs, threads } = props;
  const { launcher } = view;
  const pane = useRecordPane();
  const head = useBuildHead(view, boardKey, threads);
  const title = buildTitle(launcher);
  return (
    <Card data-launcher={launcher.name} role="group" aria-label={`Build ${head.code}: ${title}`} edge="border" edgeTone={launcher.current ? 'success' : undefined}>
      <RecordCard
        head={head}
        title={title}
        openLabel={`Open ${head.code}: ${title}`}
        onOpen={() => pane.open(head.code)}
        signals={<BuildSignals view={view} code={head.code} />}
        foot={<BuildFoot view={view} boardKey={boardKey} runs={runs} />}
      />
    </Card>
  );
};

export { LauncherCard };
