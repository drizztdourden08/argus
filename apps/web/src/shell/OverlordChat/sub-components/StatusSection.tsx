/* @layer renderer-app @kind component */
import { Button, Callout, Flex, SettingsSection, Stack, Status, Tag } from '@drizztdourden08/tessera';
import { useNow } from '../../../screens/overview/clock';
import { controlsFor } from '../behavior/controls-for';
import { uptimeText } from '../behavior/uptime-text';
import { NO_MODE, STATE_STATUSES } from '../OverlordPanel.constants';
import type { StatusSectionProps } from '../OverlordPanel.type';
import { KeyButton } from './KeyButton';

const StatusSection = (props: StatusSectionProps) => {
  const { status, action, onControl } = props;
  const now = useNow();
  const can = controlsFor(status.status);
  const facts = [status.pid ? `pid ${status.pid}` : '', status.startedAt ? `up ${uptimeText(status.startedAt, now)}` : '', status.problem ?? '']
    .filter(Boolean)
    .join(', ');
  const control = (
    <Flex gap="sm" align="center" wrap>
      <Status map={STATE_STATUSES} value={status.status} variant="pill" dot role="status" />
      <Tag>{status.mode ?? NO_MODE}</Tag>
      <Button size="sm" disabled={!can.restart || action.busy} onClick={() => onControl('restart')}>Restart</Button>
      {status.problem && <KeyButton note={status.problem} />}
    </Flex>
  );
  const row = { id: 'status', title: 'State', description: facts || undefined, input: { kind: 'custom' as const, control } };
  return (
    <Stack gap="sm">
      <SettingsSection title="Status" rows={[row]} />
      {action.problem && <Callout tone="danger">{action.problem}</Callout>}
    </Stack>
  );
};

export { StatusSection };
