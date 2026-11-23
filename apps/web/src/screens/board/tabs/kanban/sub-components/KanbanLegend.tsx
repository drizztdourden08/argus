/* @layer renderer-app @kind component */
import { useState } from 'react';
import { Flex, Icon, IconButton, Spacer, Table, TableSection } from '@drizztdourden08/tessera';
import { ToolToggle } from '../../../BoardTabsRow';
import type { KanbanState } from '../kanban.type';
import { AttentionRows } from './AttentionRows';
import { TypeRows } from './TypeRows';

const KanbanLegend = ({ state }: { state: KanbanState }) => {
  const [open, setOpen] = useState(false);
  const { compact, epicsView, setCompact, setEpicsView } = state;
  return (
    <Flex direction="column" gap="xs">
      <Flex gap="sm" align="center">
        <TypeRows />
        <Spacer />
        <ToolToggle label="Compact" pressed={compact} onToggle={() => setCompact(!compact)} />
        <ToolToggle label="Epics" pressed={epicsView} onToggle={() => setEpicsView(!epicsView)} />
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
