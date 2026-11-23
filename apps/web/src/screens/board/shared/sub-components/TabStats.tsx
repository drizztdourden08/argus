/* @layer renderer-app @kind component */
import { Flex, Tag } from '@drizztdourden08/tessera';
import { tagLook } from '../model';
import type { TabStatsProps } from '../shared.type';

const TabStats = (props: TabStatsProps) => (
  <Flex className="tab-stats" align="center" gap="xs" role="group" aria-label={props.label ?? 'Counts'}>
    {props.stats.map((stat) => <Tag key={stat.id} {...tagLook(stat.tone)} title={stat.tip}>{`${stat.label} ${stat.count}`}</Tag>)}
  </Flex>
);

export { TabStats };
