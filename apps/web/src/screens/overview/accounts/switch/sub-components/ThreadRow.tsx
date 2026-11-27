/* @layer renderer-app @kind component */
import { Box, Button, Stack, Status, Text } from '@drizztdourden08/tessera';
import { useSwitchActions } from '../behavior/useSwitchActions';
import { SKIP_THREAD_LABEL, THREAD_STATUSES, VIA_TEXT } from '../switch.constants';
import type { ThreadRowProps } from '../switch.type';

const ThreadRow = (props: ThreadRowProps) => {
  const { thread, skippable } = props;
  const actions = useSwitchActions();
  const name = thread.title ?? thread.thread ?? '';
  const pending = thread.state === undefined || thread.state === 'found' || thread.state === 'asked';
  const { board, thread: id } = thread;
  return (
    <Box as="li" className="process-list__row">
      <Status map={THREAD_STATUSES} value={thread.state} fallback="found" size="sm" dot />
      <Text variant="caption" tone="muted" mono>{board ?? ''}</Text>
      <Stack gap="2xs" className="process-list__text">
        <Text clamp={1} title={name}>{name}</Text>
        {thread.note && <Text variant="caption" tone="muted">{thread.note}</Text>}
        {thread.via && <Text variant="caption" tone="muted">{VIA_TEXT[thread.via]}</Text>}
      </Stack>
      {skippable && pending && board && id && (
        <Button size="xs" variant="ghost" disabled={actions.busy} onClick={() => actions.skipThread(board, id)}>{SKIP_THREAD_LABEL}</Button>
      )}
    </Box>
  );
};

export { ThreadRow };
