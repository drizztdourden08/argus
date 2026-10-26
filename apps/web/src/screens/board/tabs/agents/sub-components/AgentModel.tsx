/* @layer renderer-app @kind component */
import { Tag, Text, Tooltip } from '@drizztdourden08/tessera';
import type { AgentModelProps } from '../AgentsTab.type';
import { MODEL_COLORS, modelFamily, modelTip } from '../model';

const AgentModel = (props: AgentModelProps) => {
  const { agent } = props;
  if (!agent.model) return <Text tone="muted">-</Text>;
  const family = modelFamily(agent.model);
  return (
    <Tooltip content={modelTip(agent)}>
      <Tag variant="normal" color={MODEL_COLORS[family]}>{family === 'other' ? agent.model : family}</Tag>
    </Tooltip>
  );
};

export { AgentModel };
