/* @layer renderer-app @kind component */
import { Box, Card, Flex, RecordHeader, Tag } from '@drizztdourden08/tessera';
import { useId, useState } from 'react';
import { NO_LINK_FLAG } from '../records.constants';
import type { RecordGroupProps } from '../record-rows.type';
import './RecordGroup.css';

const RecordGroup = (props: RecordGroupProps) => {
  const { header, count, flagged = false, defaultOpen = true, label, children } = props;
  const [open, setOpen] = useState(defaultOpen);
  const bodyId = useId();
  return (
    <Card flush role="group" aria-label={label}>
      <Box className="record-group__head">
        <RecordHeader
          density="row"
          title={header}
          chevron={{ expanded: open, onToggle: () => setOpen(!open), controls: bodyId, label: `${open ? 'Collapse' : 'Expand'} ${label}` }}
          content={(
            <>
              {flagged && <Tag variant="urgency" color="warning" title={NO_LINK_FLAG}>no link</Tag>}
              <Tag>{count}</Tag>
            </>
          )}
        />
      </Box>
      {open && (
        <Flex id={bodyId} direction="column" gap="xs" role="list" aria-label={label}>
          {children}
        </Flex>
      )}
    </Card>
  );
};

export { RecordGroup };
