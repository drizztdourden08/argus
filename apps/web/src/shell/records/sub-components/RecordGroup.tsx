/* @layer renderer-app @kind component */
import { Flex, Tag } from '@drizztdourden08/tessera';
import { NO_LINK_FLAG } from '../records.constants';
import type { RecordGroupProps } from '../record-rows.type';
import { RecordFrame } from './RecordFrame';

const RecordGroup = (props: RecordGroupProps) => {
  const { header, count, flagged = false, defaultOpen = true, label, children } = props;
  return (
    <RecordFrame
      name={label}
      title={header}
      pinTitle
      defaultOpen={defaultOpen}
      content={
        <>
          {flagged && <Tag variant="urgency" color="warning" title={NO_LINK_FLAG}>no link</Tag>}
          <Tag>{count}</Tag>
        </>
      }
    >
      <Flex direction="column" gap="xs" role="list" aria-label={label}>
        {children}
      </Flex>
    </RecordFrame>
  );
};

export { RecordGroup };
