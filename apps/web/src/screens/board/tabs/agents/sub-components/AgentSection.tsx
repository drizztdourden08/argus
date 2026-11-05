/* @layer renderer-app @kind component */
import { Box, Text } from '@drizztdourden08/tessera';
import type { AgentSectionProps } from '../row-props.type';
import { AgentGroupList } from './AgentGroupList';
import { AgentKeyGroups } from './AgentKeyGroups';

const AgentSection = (props: AgentSectionProps) => {
  const { title, group, ...list } = props;
  return (
    <Box as="section" aria-label={title} className="agents-section">
      <Text as="p" variant="overline" className="agents-group">{`${title} · ${list.agents.length}`}</Text>
      {group === 'item' ? <AgentGroupList {...list} /> : <AgentKeyGroups group={group} {...list} />}
    </Box>
  );
};

export { AgentSection };
