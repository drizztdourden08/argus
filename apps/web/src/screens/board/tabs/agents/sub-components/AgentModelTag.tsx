/* @layer renderer-app @kind component */
import { Box, Flex, Status, Tag, Tooltip } from '@drizztdourden08/tessera';
import type { AgentModelProps } from '../row-props.type';
import { MODEL_LOOKS, modelFamily, modelTip, tagLook } from '../model';

const AgentModelTag = (props: AgentModelProps) => {
  const { agent } = props;
  if (!agent.model) return <Box as="span" className="agents-na">-</Box>;
  const family = modelFamily(agent.model);
  const look = MODEL_LOOKS[family];
  const label = family === 'other' ? agent.model : family;
  return (
    <Tooltip content={modelTip(agent)}>
      <Tag {...tagLook(look.tone)}>
        {look.dot ? <Flex inline gap="xs" align="center"><Status tone="neutral" dot aria-hidden>{null}</Status>{label}</Flex> : label}
      </Tag>
    </Tooltip>
  );
};

export { AgentModelTag };
