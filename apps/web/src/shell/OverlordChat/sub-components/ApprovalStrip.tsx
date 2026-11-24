/* @layer renderer-app @kind component */
import { Button, Callout, Flex, Stack, Text } from '@drizztdourden08/tessera';
import { Age } from '../../Age';
import type { ApprovalStripProps } from '../OverlordChat.type';

const ApprovalStrip = (props: ApprovalStripProps) => {
  const { approvals, action, onDecide } = props;
  const [first] = approvals;
  if (!first) return null;
  const buttons = (
    <Flex gap="sm" wrap>
      <Button variant="primary" size="sm" disabled={action.busy} onClick={() => onDecide(first.id, 'allow')}>Approve</Button>
      <Button size="sm" disabled={action.busy} onClick={() => onDecide(first.id, 'deny')}>Deny</Button>
    </Flex>
  );
  return (
    <Stack gap="sm">
      <Callout tone="warning" title={`Approval 1 of ${approvals.length}`} action={buttons}>
        <Stack gap="2xs">
          <Text variant="label">{first.tool}</Text>
          <Text variant="body">{first.inputSummary}</Text>
          <Text variant="caption" tone="muted">Asked <Age value={first.at} ago />. It is denied by itself when nobody answers in time.</Text>
        </Stack>
      </Callout>
      {action.problem && <Callout tone="danger">{action.problem}</Callout>}
    </Stack>
  );
};

export { ApprovalStrip };
