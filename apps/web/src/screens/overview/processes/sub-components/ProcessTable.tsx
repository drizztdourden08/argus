/* @layer renderer-app @kind component */
import { DataTable } from '@drizztdourden08/tessera';
import { PROC_COLUMNS, PROC_SCHEMA } from '../model';
import type { ProcRow } from '../model';
import { NONE_SHOWN } from '../processes.constants';
import type { ProcessTableProps } from '../processes.type';

const rowId = (row: ProcRow): string => row.id;

const ProcessTable = (props: ProcessTableProps) => {
  const { rows, selectedId, onSelect } = props;
  return (
    <DataTable
      rows={rows}
      schema={PROC_SCHEMA}
      getRowId={rowId}
      fallbackColumns={PROC_COLUMNS}
      selectedId={selectedId}
      onSelect={onSelect}
      countLabel={['process', 'processes']}
      emptyMessage={NONE_SHOWN}
    />
  );
};

export { ProcessTable };
