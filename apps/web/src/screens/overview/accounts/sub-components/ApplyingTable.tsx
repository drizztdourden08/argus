/* @layer renderer-app @kind component */
import { useMemo } from 'react';
import type { ApplyingTableProps } from '../compare.type';
import { rowMarks, visibleRows } from '../model';
import { HIDE_QUIET } from './ApplyingTable.constants';
import { CompareTable } from './CompareTable';

const ApplyingTable = (props: ApplyingTableProps) => {
  const { data, steps } = props;
  const rows = useMemo(() => visibleRows(data?.rows ?? [], HIDE_QUIET), [data]);
  const marks = useMemo(() => rowMarks(data?.rows ?? [], steps), [data, steps]);
  if (!data || rows.length === 0) return null;
  return <CompareTable accounts={data.accounts} rows={rows} marks={marks} />;
};

export { ApplyingTable };
