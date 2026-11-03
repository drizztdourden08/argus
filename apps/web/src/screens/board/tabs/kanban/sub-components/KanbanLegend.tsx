/* @layer renderer-app @kind component */
import { Disclosure, Table, TableSection } from '@drizztdourden08/tessera';
import { AttentionRows } from './AttentionRows';
import { TypeRows } from './TypeRows';

const KanbanLegend = () => (
  <Disclosure size="sm" summary="Legend">
    <Table>
      <TableSection>
        <TypeRows />
        <AttentionRows />
      </TableSection>
    </Table>
  </Disclosure>
);

export { KanbanLegend };
