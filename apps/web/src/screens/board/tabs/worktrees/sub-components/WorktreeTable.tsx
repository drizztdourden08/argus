/* @layer renderer-app @kind component */
import { DataTable } from '@drizztdourden08/tessera';
import { useMemo } from 'react';
import { tableRow } from '../model';
import type { TableRow } from '../model';
import { CELLS } from '../cells.constants';
import { FALLBACK_COLUMNS, SCHEMA } from '../worktrees.constants';
import type { RowContext } from '../worktrees.type';
import type { WorktreeTableProps } from '../worktrees-props.type';
import { WorktreeDetail } from './WorktreeDetail';

const WorktreeTable = (props: WorktreeTableProps) => {
  const { rows, meta, controls, ask, removal } = props;
  const { panel } = controls;
  const items = useMemo(() => rows.map(tableRow), [rows]);
  const busy = ask.busy || removal.busy;
  const context: RowContext = { panel, busy, meta, onAsk: controls.toggle };
  return (
    <DataTable<TableRow>
      rows={items}
      schema={SCHEMA}
      fallbackColumns={FALLBACK_COLUMNS}
      rowHeight="size-56"
      footer={false}
      headerCase="upper"
      getRowId={(item) => item.id}
      rowState={(item) => (item.source.status === 'deleting' ? 'pending' : undefined)}
      renderCell={(item, path) => {
        const Cell = CELLS[path];
        return Cell ? <Cell row={item.source} context={context} /> : undefined;
      }}
      detailRowId={panel?.id ?? null}
      renderDetail={(item) => <WorktreeDetail row={item.source} meta={meta} controls={controls} ask={ask} removal={removal} />}
      onDetailClose={() => {
        if (!busy) controls.close();
      }}
    />
  );
};

export { WorktreeTable };
