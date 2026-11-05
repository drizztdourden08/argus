/* @layer renderer-app @kind component */
import { Flex, Tag } from '@drizztdourden08/tessera';
import { RecordCard, ShownCode } from '../../../../../shell';
import { rowKeys } from '../behavior/row-keys';
import { useCaseHead } from '../behavior/useCaseHead';
import type { TestPartProps } from '../tests-props.type';
import { CaseOutcome } from './CaseOutcome';
import { ResultComposer } from './ResultComposer';

const CaseCard = (props: TestPartProps) => {
  const { test } = props;
  const view = useCaseHead(test);
  const { controls, toRun, head } = view;
  const chips = (
    <Flex gap="xs" wrap>
      {view.chips.map((chip) => <Tag key={chip.id} {...chip.look} title={chip.title}>{chip.code ? <ShownCode id={chip.text} board={head.boardKey} /> : chip.text}</Tag>)}
    </Flex>
  );
  return (
    <RecordCard
      head={head}
      title={view.title || test.id}
      openLabel={`Open ${test.id}: ${view.title || test.id}`}
      onOpen={view.open}
      signals={chips}
      attrs={{ 'data-swap-item': '', 'data-test': test.id, ...(toRun ? { 'data-to-run': '' } : {}) }}
      onKeyDown={rowKeys(toRun, controls)}
      foot={(
        <Flex direction="column" gap="xs">
          <CaseOutcome test={test} view={view} />
          {controls.open && <ResultComposer testId={test.id} outcome={controls.open} controls={controls} />}
        </Flex>
      )}
    />
  );
};

export { CaseCard };
