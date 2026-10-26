/* @layer renderer-app @kind component */
import { Card, Flex } from '@drizztdourden08/tessera';
import type { LauncherCardProps } from '../builds-props.type';
import { LauncherCommand } from './LauncherCommand';
import { LauncherHead } from './LauncherHead';
import { LauncherMeta } from './LauncherMeta';
import { LauncherNote } from './LauncherNote';
import { RunTerminal } from './RunTerminal';

const LauncherCard = (props: LauncherCardProps) => {
  const { view, boardKey, runs, onLog } = props;
  const { launcher } = view;
  const label = `Launcher ${launcher.name}${launcher.title ? `: ${launcher.title}` : ''}`;
  return (
    <Card data-launcher={launcher.name} role="group" aria-label={label}>
      <Flex direction="column" gap="sm">
        <LauncherHead view={view} />
        <LauncherMeta view={view} />
        <LauncherNote view={view} />
        <LauncherCommand view={view} boardKey={boardKey} runs={runs} onLog={onLog} />
        {view.playable && view.shown && <RunTerminal key={view.shown.id} run={view.shown} boardKey={boardKey} onLog={onLog} />}
      </Flex>
    </Card>
  );
};

export { LauncherCard };
