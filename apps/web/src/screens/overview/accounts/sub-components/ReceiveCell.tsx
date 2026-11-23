/* @layer renderer-app @kind component */
import { Box, Stack, Status, Text } from '@drizztdourden08/tessera';
import { AT_LIMIT, limitUntil, RECEIVE_STATUSES } from '../accounts.constants';
import type { ReceiveCellProps } from '../accounts.type';

const ReceiveCell = (props: ReceiveCellProps) => {
  const { receive, full, until } = props;
  if (!full) return <Box className="accounts-strip__cell"><Status map={RECEIVE_STATUSES} value={receive ? 'open' : 'locked'} size="sm" dot /></Box>;
  return (
    <Stack gap="2xs" className="accounts-strip__cell">
      <Status tone="danger" size="sm" dot>{AT_LIMIT}</Status>
      {until && <Text variant="caption" tone="muted">{limitUntil(until)}</Text>}
    </Stack>
  );
};

export { ReceiveCell };
