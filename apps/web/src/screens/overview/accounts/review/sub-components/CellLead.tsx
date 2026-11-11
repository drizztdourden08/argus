/* @layer renderer-app @kind component */
import { Checkbox, DropdownMenu } from '@drizztdourden08/tessera';
import { rowMenu } from '../model';
import type { CellLeadProps } from '../review-props.type';

const CellLead = (props: CellLeadProps) => {
  const { row, account, checked, mixed, env } = props;
  const members = env.members.get(account)?.get(row.id) ?? [];
  const cell = row.cells[account] ?? {};
  return (
    <>
      <Checkbox
        size="sm"
        ariaLabel={`Select ${row.title}`}
        checked={checked}
        indeterminate={mixed}
        disabled={row.kind === 'folder' && members.length === 0}
        onChange={(on) => env.onSelect(account, row.kind === 'folder' ? members : [row.id], on)}
      />
      <DropdownMenu
        trigger={{ label: `Actions for ${row.title}`, icon: 'ellipsis-vertical', iconOnly: true }}
        size="2xs"
        groups={rowMenu(row, cell, account, env.onAct)}
      />
    </>
  );
};

export { CellLead };
