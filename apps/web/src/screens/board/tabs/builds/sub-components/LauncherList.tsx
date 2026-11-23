/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import type { ThreadChoice } from '../../../../../data/kanban';
import { useCollection } from '../../../../../data/react';
import type { LauncherListProps } from '../builds-props.type';
import { LauncherCard } from './LauncherCard';

const LauncherList = (props: LauncherListProps) => {
  const { views, label, boardKey, runs, compact } = props;
  const threads = useCollection<ThreadChoice>(boardKey, 'threads');
  return (
    <Flex direction="column" gap="sm" role="group" aria-label={label}>
      {views.map((view) => <LauncherCard key={view.launcher.name} view={view} boardKey={boardKey} runs={runs} compact={compact} threads={threads.docs} />)}
    </Flex>
  );
};

export { LauncherList };
