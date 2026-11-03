/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import type { BuildGroupProps } from '../builds-props.type';
import { LauncherList } from './LauncherList';

const BuildGroup = (props: BuildGroupProps) => {
  const { id, head, views, label, boardKey, runs, onLog } = props;
  return (
    <Flex direction="column" gap="sm" data-group={id}>
      {head}
      <LauncherList views={views} label={label} boardKey={boardKey} runs={runs} onLog={onLog} />
    </Flex>
  );
};

export { BuildGroup };
