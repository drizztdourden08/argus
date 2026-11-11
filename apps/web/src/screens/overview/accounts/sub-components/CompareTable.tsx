/* @layer renderer-app @kind component */
import { Table, TableCell, TableRow, TableSection } from '@drizztdourden08/tessera';
import type { CompareTableProps } from '../compare.type';
import { CompareCellView } from './CompareCellView';
import { RowMarkCell } from './RowMarkCell';

const CompareTable = (props: CompareTableProps) => {
  const { accounts, rows, marks } = props;
  return (
    <Table aria-label="Account comparison">
      <TableSection head>
        <TableRow>
          {marks && <TableCell header>Step</TableCell>}
          {accounts.map((account) => <TableCell key={account.key} header>{account.label ?? account.key}</TableCell>)}
        </TableRow>
      </TableSection>
      <TableSection>
        {rows.map((row) => (
          <TableRow key={row.id} aria-current={marks?.get(row.id) === 'running' ? 'step' : undefined}>
            {marks && <RowMarkCell mark={marks.get(row.id)} />}
            {accounts.map((account) => (
              <TableCell key={account.key}><CompareCellView row={row} cell={row.cells[account.key]} /></TableCell>
            ))}
          </TableRow>
        ))}
      </TableSection>
    </Table>
  );
};

export { CompareTable };
