/* @layer renderer-app @kind component */
import { Disclosure, Flex, Tag } from '@drizztdourden08/tessera';
import { NO_LINK_FLAG } from '../records.constants';
import type { RecordGroupProps } from '../record-rows.type';

const RecordGroup = (props: RecordGroupProps) => {
  const { header, count, flagged = false, defaultOpen = true, label, children } = props;
  const end = (
    <>
      {flagged && <Tag variant="urgency" color="warning" title={NO_LINK_FLAG}>no link</Tag>}
      <Tag>{count}</Tag>
    </>
  );
  return (
    <Disclosure summary={header} end={end} defaultOpen={defaultOpen}>
      <Flex direction="column" gap="xs" role="list" aria-label={label}>
        {children}
      </Flex>
    </Disclosure>
  );
};

export { RecordGroup };
