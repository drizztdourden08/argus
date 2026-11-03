/* @layer renderer-app @kind component */
import { Disclosure, Flex, Tag } from '@drizztdourden08/tessera';
import { NO_LINK_FLAG } from '../records.constants';
import type { RecordGroupProps } from '../records.type';

const RecordGroup = (props: RecordGroupProps) => {
  const { header, count, flagged = false, defaultOpen = true, label, children } = props;
  return (
    <Disclosure
      summary={header}
      end={
        <Flex gap="xs" align="center">
          {flagged && <Tag variant="urgency" color="warning" title={NO_LINK_FLAG}>no link</Tag>}
          <Tag>{count}</Tag>
        </Flex>
      }
      defaultOpen={defaultOpen}
    >
      <Flex direction="column" gap="xs" role="list" aria-label={label}>
        {children}
      </Flex>
    </Disclosure>
  );
};

export { RecordGroup };
