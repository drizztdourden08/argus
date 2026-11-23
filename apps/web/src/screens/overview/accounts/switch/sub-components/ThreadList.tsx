/* @layer renderer-app @kind component */
import { Box, Inline, Stack, Status, Text } from '@drizztdourden08/tessera';
import { readyText } from '../model';
import { NO_THREADS, THREAD_STATUSES, THREADS_TITLE, THREADS_UNCHECKED } from '../switch.constants';
import type { ThreadListProps } from '../switch.type';

const ThreadList = (props: ThreadListProps) => {
  const { threads, checked } = props;
  const empty = checked ? NO_THREADS : THREADS_UNCHECKED;
  return (
    <Stack gap="2xs" as="section" aria-label={THREADS_TITLE}>
      <Inline gap="sm" align="center">
        <Text variant="overline" tone="muted">{THREADS_TITLE}</Text>
        {threads.length > 0 && <Text variant="caption" tone="muted">{readyText(threads)}</Text>}
      </Inline>
      {threads.length === 0 && <Text variant="caption" tone="faint">{empty}</Text>}
      {threads.length > 0 && (
        <Box as="ul" className="process-list">
          {threads.map((thread, index) => (
            <Box as="li" key={thread.sessionId ?? `${thread.board ?? ''}:${thread.thread ?? index}`} className="process-list__row">
              <Status map={THREAD_STATUSES} value={thread.state} fallback="found" size="sm" dot />
              <Text variant="caption" tone="muted" mono>{thread.board ?? ''}</Text>
              <Stack gap="2xs" className="process-list__text">
                <Text clamp={1} title={thread.title ?? thread.thread ?? ''}>{thread.title ?? thread.thread ?? ''}</Text>
                {thread.note && <Text variant="caption" tone="muted">{thread.note}</Text>}
              </Stack>
            </Box>
          ))}
        </Box>
      )}
    </Stack>
  );
};

export { ThreadList };
