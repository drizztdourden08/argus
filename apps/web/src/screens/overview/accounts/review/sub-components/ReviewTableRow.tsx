/* @layer renderer-app @kind component */
import { TableCell, TableRow } from '@drizztdourden08/tessera';
import { choiceOf } from '../model';
import type { ReviewTableRowProps } from '../review-props.type';
import { ChoiceChip } from './ChoiceChip';
import { TreeCell } from './TreeCell';

const ReviewTableRow = (props: ReviewTableRowProps) => {
  const { accounts, row, env } = props;
  return (
    <TableRow>
      {accounts.map((account, index) => [
        <TableCell key={account.key}><TreeCell row={row} account={account} env={env} /></TableCell>,
        index < accounts.length - 1 && (
          <TableCell key={`${account.key}-mid`} align="center" className="review-mid">
            {index === 0 && row.diff && <ChoiceChip diff={row.diff} choice={choiceOf(row, env.decisions)} onChoose={(choice) => env.onChoose(row.id, choice, row.diff?.default ?? choice)} />}
          </TableCell>
        ),
      ])}
    </TableRow>
  );
};

export { ReviewTableRow };
