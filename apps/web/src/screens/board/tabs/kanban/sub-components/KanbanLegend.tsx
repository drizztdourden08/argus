/* @layer renderer-app @kind component */
import { Disclosure, Table, TableSection } from '@drizztdourden08/tessera';
import { AttentionRows } from './AttentionRows';

const KanbanLegend = () => (
  <Disclosure size="sm" summary="Legend">
    <Table>
      <TableSection>
        <AttentionRows />
      </TableSection>
    </Table>
  </Disclosure>
);

export { KanbanLegend };
