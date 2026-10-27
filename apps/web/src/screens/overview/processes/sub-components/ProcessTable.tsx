/* @layer renderer-app @kind component */
import { DataTable } from '@drizztdourden08/tessera';
import { CUSTOM_PATHS, PROC_COLUMNS, PROC_SCHEMA, rowState } from '../model';
import type { ProcRow } from '../model';
import { NONE_SHOWN } from '../processes.constants';
import type { ProcessTableProps } from '../processes.type';
import { KillRowConfirm } from './KillRowConfirm';
import { ProcessCell } from './ProcessCell';

const rowId = (row: ProcRow): string => row.id;

const ProcessTable = (props: ProcessTableProps) => {
  const { rows, kill } = props;
  return (
    <DataTable
      rows={rows}
      schema={PROC_SCHEMA}
      getRowId={rowId}
      rowHeight="size-28"
      footer={false}
      headerCase="upper"
      rowState={rowState}
      fallbackColumns={PROC_COLUMNS}
      renderCell={(row, path) => (CUSTOM_PATHS.includes(path) ? <ProcessCell row={row} path={path} kill={kill} /> : undefined)}
      detailRowId={kill.ask?.kind === 'proc' ? kill.ask.id : null}
      renderDetail={(row) => <KillRowConfirm row={row} kill={kill} />}
      onDetailClose={kill.cancel}
      emptyMessage={NONE_SHOWN}
    />
  );
};

export { ProcessTable };
