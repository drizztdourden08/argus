/* @layer renderer-app @kind component */
import { EmptyState } from '@drizztdourden08/tessera';
import { EMPTY_COMPARE } from '../compare.constants';
import type { CompareBodyProps } from '../compare.type';
import { CompareSummary } from './CompareSummary';
import { CompareTable } from './CompareTable';

const CompareBody = (props: CompareBodyProps) => {
  const { data, rows } = props;
  return (
    <>
      <CompareSummary rows={data.rows} same={data.counts?.same ?? 0} />
      {rows.length === 0 ? <EmptyState message={EMPTY_COMPARE} /> : <CompareTable accounts={data.accounts} rows={rows} />}
    </>
  );
};

export { CompareBody };
