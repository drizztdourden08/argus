/* @layer renderer-app @kind component */
import { Flex, Icon, Tag, Text } from '@drizztdourden08/tessera';
import { formatAgo } from '../../../../../format';
import type { AgentNowProps } from '../AgentsTab.type';
import { agentNow, isStalled } from '../model';

const quietSince = (props: AgentNowProps): string => {
  const { agent } = props;
  return agent.live?.lastActivityAt ?? agent.startedAt ?? agent.updatedAt ?? '';
};

const AgentNow = (props: AgentNowProps) => {
  const { agent, now } = props;
  const doing = agentNow(agent);
  const stalled = isStalled(agent);
  if (!doing && !stalled) return <Text as="span" tone="muted">-</Text>;
  const age = formatAgo(quietSince(props), now);
  return (
    <>
      {stalled && (
        <Tag variant="urgency" color="warning" title={`No new line in the transcript for ${age} (it counts as stalled after a few minutes of quiet)`}>
          <Flex inline gap="xs" align="center"><Icon name="triangle-alert" size={11} aria-hidden />{`stalled ${age}`}</Flex>
        </Tag>
      )}
      {doing && ` ${doing.text}${doing.since ? ` · ${formatAgo(doing.since, now)}` : ''}`}
    </>
  );
};

export { AgentNow };
