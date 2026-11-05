/* @layer renderer-app @kind component */
import { Button, EmptyState, Flex, Text } from '@drizztdourden08/tessera';
import { useLogWindow } from '../behavior/useLogWindow';
import { LOG_START, LOG_STEP } from '../OwnerFeed.constants';
import type { LogSectionProps } from '../OwnerFeedLog.type';
import { LogList } from './LogList';

const LogSection = (props: LogSectionProps) => {
  const { lines } = props;
  const log = useLogWindow(lines.length);
  if (lines.length === 0) return <EmptyState message="Nothing logged yet." size="sm" />;
  return (
    <Flex direction="column" gap="sm">
      <Text variant="overline" tone="muted">Work log</Text>
      <LogList lines={lines.slice(0, log.shown)} />
      {log.left > 0 ? (
        <Button size="sm" variant="ghost" onClick={log.more}>
          Show {Math.min(LOG_STEP, log.left)} more ({log.left} left)
        </Button>
      ) : null}
      {log.shown > LOG_START ? <Button size="sm" variant="ghost" onClick={log.less}>Show less</Button> : null}
    </Flex>
  );
};

export { LogSection };
