/* @layer renderer-app @kind component */
import { useMemo } from 'react';
import type { ApplyingTableProps } from '../compare.type';
import { rowMarks } from '../model';
import { visibleRows } from '../review';
import { NO_COLLAPSED } from './ApplyingTable.constants';
import { CompareTable } from './CompareTable';

const ApplyingTable = (props: ApplyingTableProps) => {
  const { data, steps } = props;
  const rows = useMemo(() => visibleRows(data?.rows ?? [], 'differences', NO_COLLAPSED), [data]);
  const marks = useMemo(() => rowMarks(data?.rows ?? [], steps), [data, steps]);
  if (!data || rows.length === 0) return null;
  return <CompareTable accounts={data.accounts} rows={rows} marks={marks} />;
};

export { ApplyingTable };
