/* @layer renderer-app @kind component */
import { Disclosure, EmptyState, Flex } from '@drizztdourden08/tessera';
import type { LogFoldProps } from '../OwnerFeedLog.type';
import { LogSection } from './LogSection';
import { NoticeList } from './NoticeList';

const LogFold = (props: LogFoldProps) => {
  const { filter, notices, lines, ctx } = props;
  const total = notices.length + lines.length;
  if (total === 0) return filter === 'log' ? <EmptyState message="Nothing logged yet." size="sm" /> : null;
  return (
    <Disclosure key={filter} summary={`Log · ${total}`} defaultOpen={filter === 'log'}>
      <Flex direction="column" gap="md">
        {notices.length > 0 ? <NoticeList notices={notices} ctx={ctx} /> : null}
        {lines.length > 0 ? <LogSection lines={lines} /> : null}
      </Flex>
    </Disclosure>
  );
};

export { LogFold };
