/* @layer renderer-app @kind component */
import { Inline, Text } from '@drizztdourden08/tessera';
import { STATUS_LOOK } from '../compare.constants';
import type { CompareSummaryProps } from '../compare.type';
import { statusCounts } from '../model';
import { StatusChip } from './StatusChip';

const CompareSummary = (props: CompareSummaryProps) => {
  const counts = [...statusCounts(props.rows)];
  return (
    <Inline gap="sm" align="center" wrap>
      <StatusChip look={{ label: `${props.same} same`, tone: 'success' }} />
      {counts.map(([status, count]) => (
        <StatusChip key={status} look={{ label: `${count} ${STATUS_LOOK[status].label}`, tone: STATUS_LOOK[status].tone }} />
      ))}
      {counts.length === 0 && <Text variant="subtitle">No differences</Text>}
    </Inline>
  );
};

export { CompareSummary };
