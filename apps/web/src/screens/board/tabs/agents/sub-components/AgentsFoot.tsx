/* @layer renderer-app @kind component */
import { Button, Flex, Text } from '@drizztdourden08/tessera';
import { FINISHED_HINT } from '../AgentsTab.constants';
import type { AgentsFootProps } from '../AgentsTab.type';

const AgentsFoot = (props: AgentsFootProps) => {
  const { canArchive, onArchive } = props;
  return (
    <Flex className="agents-foot" align="center" gap="sm" wrap>
      {canArchive && <Button size="xs" variant="ghost" onClick={onArchive}>Archive finished</Button>}
      <Text as="span" variant="caption">{FINISHED_HINT}</Text>
    </Flex>
  );
};

export { AgentsFoot };
