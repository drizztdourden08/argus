/* @layer renderer-app @kind component */
import { Box, Button, Flex, Stack, Text } from '@drizztdourden08/tessera';
import { Age } from '../../Age';
import type { ApprovalLineProps } from '../OverlordChat.type';

const ApprovalLine = (props: ApprovalLineProps) => {
  const { approval, busy, onDecide } = props;
  return (
    <Box surface="sunken">
      <Stack gap="sm">
        <Text variant="label">{approval.tool}</Text>
        <Text variant="body">{approval.inputSummary}</Text>
        <Text variant="caption" tone="muted">Asked <Age value={approval.at} ago /></Text>
        <Flex gap="sm">
          <Button size="xs" variant="primary" disabled={busy} onClick={() => onDecide(approval.id, 'allow')}>Allow</Button>
          <Button size="xs" disabled={busy} onClick={() => onDecide(approval.id, 'deny')}>Deny</Button>
        </Flex>
      </Stack>
    </Box>
  );
};

export { ApprovalLine };
