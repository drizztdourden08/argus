/* @layer renderer-app @kind component */
import { DataTable } from '@drizztdourden08/tessera';
import { useMemo } from 'react';
import type { CacheTableProps } from '../cache.type';
import { CACHE_COLUMNS, CACHE_SCHEMA, CUSTOM_PATHS, toCacheRow } from '../model';
import type { CacheTableRow } from '../model';
import { CacheCell } from './CacheCell';

const rowId = (row: CacheTableRow): string => row.id;

const CacheTable = (props: CacheTableProps) => {
  const { report } = props;
  const rows = useMemo(() => report.rows.map(toCacheRow), [report.rows]);
  return (
    <DataTable
      rows={rows}
      schema={CACHE_SCHEMA}
      getRowId={rowId}
      rowHeight="size-40"
      footer={false}
      headerCase="upper"
      fallbackColumns={CACHE_COLUMNS}
      renderCell={(row, path) => (CUSTOM_PATHS.includes(path) ? <CacheCell row={row} path={path} /> : undefined)}
    />
  );
};

export { CacheTable };
