/* @layer renderer-app @kind component */
import { Button, Flex, Text } from '@drizztdourden08/tessera';
import { FINISHED_HINT } from '../AgentsTab.constants';
import type { AgentsFootProps } from '../AgentsTab.type';

const AgentsFoot = (props: AgentsFootProps) => {
  const { count, showFinished, canArchive, onToggle, onArchive } = props;
  return (
    <Flex className="agents-foot" align="center" gap="sm" wrap>
      <Button size="xs" variant="ghost" aria-pressed={showFinished} onClick={onToggle}>
        {`${showFinished ? 'Hide' : 'Show'} finished (${count})`}
      </Button>
      {showFinished && canArchive && <Button size="xs" variant="ghost" onClick={onArchive}>Archive finished</Button>}
      <Text as="span" variant="caption">{FINISHED_HINT}</Text>
    </Flex>
  );
};

export { AgentsFoot };
