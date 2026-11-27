/* @layer renderer-app @kind component */
import { Box, Status, Text } from '@drizztdourden08/tessera';
import { THREAD_STATUSES } from '../switch.constants';
import type { ThreadRowProps } from '../switch.type';
import { ThreadSkip } from './ThreadSkip';
import { ThreadText } from './ThreadText';

const ThreadRow = (props: ThreadRowProps) => (
  <Box as="li" className="process-list__row">
    <Status map={THREAD_STATUSES} value={props.thread.state} fallback="found" size="sm" dot />
    <Text variant="caption" tone="muted" mono>{props.thread.board ?? ''}</Text>
    <ThreadText thread={props.thread} />
    <ThreadSkip {...props} />
  </Box>
);

export { ThreadRow };
