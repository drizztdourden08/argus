/* @layer renderer-app @kind component */
import { Box, Inline, Stack, Text } from '@drizztdourden08/tessera';
import { readyText } from '../model';
import { NO_THREADS, THREADS_TITLE, THREADS_UNCHECKED } from '../switch.constants';
import type { ThreadListProps } from '../switch.type';
import { ThreadRow } from './ThreadRow';

const ThreadList = (props: ThreadListProps) => {
  const { threads, checked, skippable } = props;
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
            <ThreadRow key={thread.sessionId ?? `${thread.board ?? ''}:${thread.thread ?? index}`} thread={thread} skippable={skippable} />
          ))}
        </Box>
      )}
    </Stack>
  );
};

export { ThreadList };
