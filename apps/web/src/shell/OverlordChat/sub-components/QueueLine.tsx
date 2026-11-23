/* @layer renderer-app @kind component */
import { Stack, Text } from '@drizztdourden08/tessera';
import { Age } from '../../Age';
import type { QueueLineProps } from '../chat-panels.type';

const QueueLine = (props: QueueLineProps) => {
  const { text, label, at } = props;
  return (
    <Stack gap="2xs" className="overlord-chat__row">
      {label && <Text variant="label">{label}</Text>}
      <Text variant="body">{text}</Text>
      {at && <Text variant="caption" tone="muted"><Age value={at} ago /></Text>}
    </Stack>
  );
};

export { QueueLine };
