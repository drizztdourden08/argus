/* @layer renderer-app @kind component */
import { Flex, Status, Tag, Text, Tooltip } from '@drizztdourden08/tessera';
import type { AgentModelProps } from '../AgentsTab.type';
import { MODEL_LOOKS, modelFamily, modelTip, tagLook } from '../model';

const AgentModelTag = (props: AgentModelProps) => {
  const { agent } = props;
  if (!agent.model) return <Text as="span" tone="dim">-</Text>;
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
