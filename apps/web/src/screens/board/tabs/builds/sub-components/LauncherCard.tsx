/* @layer renderer-app @kind component */
import { Card } from '@drizztdourden08/tessera';
import { RecordCard, useRecordPane } from '../../../../../shell';
import { OwnerOpDialog, useOwnerAsk } from '../../../owner-links';
import { useBuildHead } from '../behavior/useBuildHead';
import type { LauncherCardProps } from '../builds-props.type';
import { buildTitle } from '../model';
import { BuildFoot } from './BuildFoot';
import { BuildSignals } from './BuildSignals';

const LauncherCard = (props: LauncherCardProps) => {
  const { view, boardKey, runs, threads } = props;
  const { launcher } = view;
  const pane = useRecordPane();
  const owner = useOwnerAsk();
  const head = useBuildHead(view, boardKey, threads, owner.open);
  const title = buildTitle(launcher);
  return (
    <Card data-swap-item data-launcher={launcher.name} role="group" aria-label={`Build ${head.code}: ${title}`} edge="border" edgeTone={launcher.current ? 'success' : undefined}>
      <RecordCard
        head={head}
        title={title}
        openLabel={`Open ${head.code}: ${title}`}
        onOpen={() => pane.open(head.code)}
        signals={<BuildSignals view={view} code={head.code} />}
        foot={<BuildFoot view={view} boardKey={boardKey} runs={runs} />}
      />
      {owner.ask && <OwnerOpDialog boardKey={boardKey} ask={owner.ask} onClose={owner.close} />}
    </Card>
  );
};

export { LauncherCard };
