/* @layer renderer-app @kind component */
import { Box, Tag, Tooltip } from '@drizztdourden08/tessera';
import type { AgentModelProps } from '../row-props.type';
import { MODEL_LOOKS, modelFamily, modelLabel, modelTip, tagLook } from '../model';

const AgentModelTag = (props: AgentModelProps) => {
  const { agent } = props;
  if (!agent.model) return <Box as="span" className="agents-na">-</Box>;
  const look = MODEL_LOOKS[modelFamily(agent.model)];
  return (
    <Tooltip content={modelTip(agent)}>
      <Tag {...tagLook(look.tone)} dot={look.dot}>{modelLabel(agent.model)}</Tag>
    </Tooltip>
  );
};

export { AgentModelTag };
