/* @layer renderer-app @kind component */
import { Stack, Text } from '@drizztdourden08/tessera';
import { VIA_TEXT } from '../switch.constants';
import type { ThreadRowProps } from '../switch.type';

const ThreadText = (props: Pick<ThreadRowProps, 'thread'>) => {
  const { thread } = props;
  const name = thread.title ?? thread.thread ?? '';
  return (
    <Stack gap="2xs" className="process-list__text">
      <Text clamp={1} title={name}>{name}</Text>
      {thread.note && <Text variant="caption" tone="muted">{thread.note}</Text>}
      {thread.via && <Text variant="caption" tone="muted">{VIA_TEXT[thread.via]}</Text>}
    </Stack>
  );
};

export { ThreadText };
