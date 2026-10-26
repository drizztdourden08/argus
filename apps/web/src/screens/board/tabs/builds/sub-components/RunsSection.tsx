/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import type { RunsSectionProps } from '../builds-props.type';
import { RunList } from './RunList';

const RunsSection = (props: RunsSectionProps) => {
  const { groups, boardKey, onLog } = props;
  return (
    <Flex direction="column" gap="md">
      {groups.live.length > 0 && <RunList title="Running now" runs={groups.live} boardKey={boardKey} onLog={onLog} />}
      {groups.finished.length > 0 && <RunList title="Finished runs" runs={groups.finished} boardKey={boardKey} onLog={onLog} />}
    </Flex>
  );
};

export { RunsSection };
