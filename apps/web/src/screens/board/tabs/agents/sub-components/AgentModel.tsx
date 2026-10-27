/* @layer renderer-app @kind component */
import type { AgentModelProps } from '../row-props.type';
import { AgentCell } from './AgentCell';
import { AgentModelTag } from './AgentModelTag';

const AgentModel = (props: AgentModelProps) => (
  <AgentCell name="model">
    <AgentModelTag agent={props.agent} />
  </AgentCell>
);

export { AgentModel };
