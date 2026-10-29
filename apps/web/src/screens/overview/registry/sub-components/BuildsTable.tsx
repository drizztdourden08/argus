/* @layer renderer-app @kind component */
import { DataTable } from '@drizztdourden08/tessera';
import { useMemo } from 'react';
import { BUILD_COLUMNS, BUILD_SCHEMA, toBuildRow } from '../model';
import type { BuildRow } from '../model';
import type { BuildsTableProps } from '../registry.type';
import { BuildCell } from './BuildCell';

const rowId = (row: BuildRow): string => row.id;

const BuildsTable = (props: BuildsTableProps) => {
  const { builds } = props;
  const rows = useMemo(() => builds.map(toBuildRow), [builds]);
  return (
    <DataTable
      rows={rows}
      schema={BUILD_SCHEMA}
      getRowId={rowId}
      rowHeight="size-40"
      footer={false}
      headerCase="upper"
      fallbackColumns={BUILD_COLUMNS}
      renderCell={(row, path) => <BuildCell row={row} path={path} />}
    />
  );
};

export { BuildsTable };
