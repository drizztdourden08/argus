/* @layer renderer-app @kind component */
import { Box, Stack, Text } from '@drizztdourden08/tessera';
import { Age } from '../../Age';
import type { QueueLineProps } from '../OverlordChat.type';

const QueueLine = (props: QueueLineProps) => {
  const { text, label, at } = props;
  return (
    <Box surface="sunken">
      <Stack gap="2xs">
        {label && <Text variant="label">{label}</Text>}
        <Text variant="body">{text}</Text>
        {at && <Text variant="caption" tone="muted"><Age value={at} ago /></Text>}
      </Stack>
    </Box>
  );
};

export { QueueLine };
