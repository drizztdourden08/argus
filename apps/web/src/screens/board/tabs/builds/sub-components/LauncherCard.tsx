/* @layer renderer-app @kind component */
import { RecordCard, useRecordPane } from '../../../../../shell';
import { OwnerOpDialog, useOwnerAsk } from '../../../owner-links';
import { useBuildHead } from '../behavior/useBuildHead';
import type { LauncherCardProps } from '../builds-props.type';
import { buildTitle } from '../model';
import { BuildFoot } from './BuildFoot';
import { BuildSignals } from './BuildSignals';

const LauncherCard = (props: LauncherCardProps) => {
  const { view, boardKey, runs, threads, compact } = props;
  const { launcher } = view;
  const pane = useRecordPane();
  const owner = useOwnerAsk();
  const head = useBuildHead(view, boardKey, threads, owner.open);
  const title = buildTitle(launcher);
  return (
    <>
      <RecordCard
        head={head}
        title={title}
        openLabel={`Open ${head.code}: ${title}`}
        onOpen={() => pane.open(head.code)}
        label={`Build ${head.code}: ${title}`}
        time={launcher.retiredAt ?? launcher.updatedAt ?? null}
        compact={compact}
        chevronOnIcon
        menuOnHover
        edge="border"
        edgeTone={launcher.current ? 'success' : undefined}
        attrs={{ 'data-swap-item': '', 'data-launcher': launcher.name }}
        signals={<BuildSignals view={view} code={head.code} />}
        foot={<BuildFoot view={view} boardKey={boardKey} runs={runs} compact={compact} />}
      />
      {owner.ask && <OwnerOpDialog boardKey={boardKey} ask={owner.ask} onClose={owner.close} />}
    </>
  );
};

export { LauncherCard };
