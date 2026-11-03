/* @layer renderer-app @kind component */
import { Box, Text } from '@drizztdourden08/tessera';
import type { AgentSectionProps } from '../row-props.type';
import { AgentGroupList } from './AgentGroupList';

const AgentSection = (props: AgentSectionProps) => {
  const { title, ...list } = props;
  return (
    <Box as="section" aria-label={title} className="agents-section">
      <Text as="p" variant="overline" className="agents-group">{`${title} \u00b7 ${list.agents.length}`}</Text>
      <AgentGroupList {...list} />
    </Box>
  );
};

export { AgentSection };
