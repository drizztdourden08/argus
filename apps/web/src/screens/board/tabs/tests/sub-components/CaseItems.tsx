/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import { useTestsShared } from '../behavior/useTestsShared';
import type { CaseListProps } from '../tests-props.type';
import { CaseCard } from './CaseCard';
import { CaseRow } from './CaseRow';

const CaseItems = (props: CaseListProps) => {
  const { tests } = props;
  const { layout } = useTestsShared();
  if (layout === 'normal') return <>{tests.map((test) => <CaseRow key={test.id} test={test} />)}</>;
  return (
    <Flex direction="column" gap="xs" className="test-section__grid">
      {tests.map((test) => <CaseCard key={test.id} test={test} />)}
    </Flex>
  );
};

export { CaseItems };
