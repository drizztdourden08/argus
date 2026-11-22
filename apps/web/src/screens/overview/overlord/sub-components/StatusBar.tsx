/* @layer renderer-app @kind component */
import { Button, Callout, Flex, Stack, Status, Tag, Text } from '@drizztdourden08/tessera';
import { KeyButton } from '../../../../shell/OverlordChat';
import { useNow } from '../behavior/useNow';
import { controlsFor, uptimeText } from '../model';
import { NO_MODE, STATE_STATUSES } from '../overlord.constants';
import type { StatusBarProps } from '../overlord.type';

const StatusBar = (props: StatusBarProps) => {
  const { status, action, onControl, chatOpen, onChat } = props;
  const now = useNow();
  const can = controlsFor(status.status);
  const facts = [status.pid ? `pid ${status.pid}` : '', status.startedAt ? `up ${uptimeText(status.startedAt, now)}` : ''].filter(Boolean).join(', ');
  return (
    <Stack gap="sm">
      <Flex gap="sm" align="center" wrap>
        <Text as="h2" variant="title">Overlord</Text>
        <Status map={STATE_STATUSES} value={status.status} variant="pill" dot role="status" />
        <Tag>{status.mode ?? NO_MODE}</Tag>
        {facts && <Text variant="caption" tone="muted">{facts}</Text>}
      </Flex>
      <Flex gap="sm" wrap>
        <Button size="sm" disabled={!can.restart || action.busy} onClick={() => onControl('restart')}>Restart</Button>
        {status.problem && <KeyButton note={status.problem} />}
        <Button size="sm" active={chatOpen} onClick={onChat}>Chat</Button>
      </Flex>
      {status.problem && <Text variant="caption" tone="muted">{status.problem}</Text>}
      {action.problem && <Callout tone="danger">{action.problem}</Callout>}
    </Stack>
  );
};

export { StatusBar };
