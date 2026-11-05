/* @layer renderer-app @kind component */
import { Box } from '@drizztdourden08/tessera';
import { RecordRow } from '../../../../../shell';
import { rowKeys } from '../behavior/row-keys';
import { useCaseHead } from '../behavior/useCaseHead';
import type { TestPartProps } from '../tests-props.type';
import { CaseOutcome } from './CaseOutcome';
import { ResultComposer } from './ResultComposer';

const CaseRow = (props: TestPartProps) => {
  const { test } = props;
  const view = useCaseHead(test);
  const { controls, toRun } = view;
  return (
    <Box data-swap-item data-test={test.id} data-to-run={toRun ? '' : undefined} onKeyDown={rowKeys(toRun, controls)}>
      <RecordRow head={view.head} title={view.title} onOpen={view.open} chips={view.chips} trailing={<CaseOutcome test={test} view={view} />} />
      {controls.open && <ResultComposer testId={test.id} outcome={controls.open} controls={controls} />}
    </Box>
  );
};

export { CaseRow };
