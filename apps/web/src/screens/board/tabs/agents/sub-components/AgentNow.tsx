/* @layer renderer-app @kind component */
import { Box, Flex, Icon, Tag } from '@drizztdourden08/tessera';
import { formatAgo } from '../../../../../format';
import type { AgentNowProps } from '../row-props.type';
import { agentNow, isStalled } from '../model';

const quietSince = (props: AgentNowProps): string => {
  const { agent } = props;
  return agent.live?.lastActivityAt ?? agent.startedAt ?? agent.updatedAt ?? '';
};

const AgentNow = (props: AgentNowProps) => {
  const { agent, now } = props;
  const doing = agentNow(agent);
  const stalled = isStalled(agent);
  if (!doing && !stalled) return <Box as="span" className="agents-na">-</Box>;
  const text = doing ? `${doing.text}${doing.since ? ` - ${formatAgo(doing.since, now)}` : ''}` : '';
  const age = formatAgo(quietSince(props), now);
  return (
    <>
      {stalled && (
        <Tag variant="urgency" color="warning" title={`No new line in the transcript for ${age} (it counts as stalled after a few minutes of quiet)`}>
          <Flex inline gap="xs" align="center"><Icon name="triangle-alert" size={11} aria-hidden />{`stalled ${age}`}</Flex>
        </Tag>
      )}
      {doing && <Box as="span" className="agents-now" title={text}>{text}</Box>}
    </>
  );
};

export { AgentNow };
