/* @layer renderer-app @kind component */
import { Table, TableCell, TableRow, TableSection } from '@drizztdourden08/tessera';
import { DEP_COLUMNS } from '../deps.constants';
import type { DepsTableProps } from '../deps.type';
import { DepRow } from './DepRow';

const DepsTable = (props: DepsTableProps) => {
  const { docs, now } = props;
  return (
    <Table aria-label="Dependencies between projects">
      <TableSection head>
        <TableRow>
          {DEP_COLUMNS.map((column) => <TableCell header key={column}>{column}</TableCell>)}
        </TableRow>
      </TableSection>
      <TableSection>
        {docs.map((doc) => <DepRow key={doc.id} doc={doc} now={now} />)}
      </TableSection>
    </Table>
  );
};

export { DepsTable };
