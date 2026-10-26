/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import type { LauncherListProps } from '../builds-props.type';
import { LauncherCard } from './LauncherCard';

const LauncherList = (props: LauncherListProps) => {
  const { views, label, boardKey, runs, onLog } = props;
  return (
    <Flex direction="column" gap="md" role="group" aria-label={label}>
      {views.map((view) => <LauncherCard key={view.launcher.name} view={view} boardKey={boardKey} runs={runs} onLog={onLog} />)}
    </Flex>
  );
};

export { LauncherList };
