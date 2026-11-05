/* @layer renderer-app @kind component */
import { Grid } from '@drizztdourden08/tessera';
import { useTestsShared } from '../behavior/useTestsShared';
import { CASE_CARD_WIDTH } from '../tests.constants';
import type { CaseListProps } from '../tests-props.type';
import { CaseCard } from './CaseCard';
import { CaseRow } from './CaseRow';

const CaseItems = (props: CaseListProps) => {
  const { tests } = props;
  const { layout } = useTestsShared();
  if (layout === 'rows') return <>{tests.map((test) => <CaseRow key={test.id} test={test} />)}</>;
  return (
    <Grid minColWidth={CASE_CARD_WIDTH} gap="sm" className="test-section__grid">
      {tests.map((test) => <CaseCard key={test.id} test={test} />)}
    </Grid>
  );
};

export { CaseItems };
