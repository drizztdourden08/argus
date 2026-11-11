/* @layer renderer-app @kind component */
import './ReviewTable.css';
import { Table, TableCell, TableRow, TableSection } from '@drizztdourden08/tessera';
import type { ReviewTableProps } from '../review-props.type';
import { ReviewTableRow } from './ReviewTableRow';

const ReviewTable = (props: ReviewTableProps) => {
  const { accounts, rows, env } = props;
  return (
    <Table aria-label="Account review">
      <TableSection head>
        <TableRow>
          {accounts.map((account, index) => [
            <TableCell key={account.key} header>{account.label ?? account.key}</TableCell>,
            index < accounts.length - 1 && <TableCell key={`${account.key}-mid`} header align="center" className="review-mid">Choice</TableCell>,
          ])}
        </TableRow>
      </TableSection>
      <TableSection>
        {rows.map((row) => <ReviewTableRow key={row.id} accounts={accounts} row={row} env={env} />)}
      </TableSection>
    </Table>
  );
};

export { ReviewTable };
