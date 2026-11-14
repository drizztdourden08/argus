/* @layer renderer-app @kind component */
import { Box, Stack, Text } from '@drizztdourden08/tessera';
import type { TalkRowViewProps } from '../overlord.type';
import { TALK_WHO } from '../overlord.constants';

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
