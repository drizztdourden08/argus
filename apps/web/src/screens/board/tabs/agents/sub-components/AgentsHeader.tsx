/* @layer renderer-app @kind component */
import { Flex, Tag } from '@drizztdourden08/tessera';
import type { AgentsHeaderProps } from '../AgentsTab.type';
import { tagLook } from '../model';

const AgentsHeader = (props: AgentsHeaderProps) => (
  <Flex className="agents-head" align="center" gap="xs" role="group" aria-label="Agent counts">
    {props.stats.map((stat) => <Tag key={stat.id} {...tagLook(stat.tone)}>{`${stat.label} ${stat.count}`}</Tag>)}
  </Flex>
);

export { AgentsHeader };
