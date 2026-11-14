/* @layer renderer-app @kind component */
import { Box, Stack, Text } from '@drizztdourden08/tessera';
import type { TalkRowViewProps } from '../OverlordChat.type';
import { TALK_WHO } from '../OverlordChat.constants';

const TalkRowView = (props: TalkRowViewProps) => {
  const { row } = props;
  return (
    <Box surface="sunken" className={`overlord-talk overlord-talk--${row.kind}`}>
      <Stack gap="2xs">
        <Text variant="overline" tone="muted">{TALK_WHO[row.kind]}</Text>
        <Text variant="body">{row.text}</Text>
      </Stack>
    </Box>
  );
};

export { TalkRowView };
