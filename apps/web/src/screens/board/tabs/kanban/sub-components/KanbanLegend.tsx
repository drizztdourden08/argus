/* @layer renderer-app @kind component */
import { useState } from 'react';
import { Flex, Icon, IconButton, Spacer, Table, TableSection } from '@drizztdourden08/tessera';
import { AttentionRows } from './AttentionRows';
import { TypeRows } from './TypeRows';

const KanbanLegend = () => {
  const [open, setOpen] = useState(false);
  return (
    <Flex direction="column" gap="xs">
      <Flex gap="sm" align="center">
        <TypeRows />
        <Spacer />
        <IconButton label={open ? 'Hide legend' : 'Show legend'} size="xs" variant="ghost" aria-expanded={open} onClick={() => setOpen(!open)}>
          <Icon name={open ? 'chevron-up' : 'chevron-down'} />
        </IconButton>
      </Flex>
      {open && (
        <Table>
          <TableSection>
            <AttentionRows />
          </TableSection>
        </Table>
      )}
    </Flex>
  );
};

export { KanbanLegend };
