/* @layer renderer-app @kind component */
import { Flex, Tag } from '@drizztdourden08/tessera';
import { RecordCard, ShownCode } from '../../../../../shell';
import { rowKeys } from '../behavior/row-keys';
import { useTestsShared } from '../behavior/useTestsShared';
import { useCaseHead } from '../behavior/useCaseHead';
import type { TestPartProps } from '../tests-props.type';
import { CaseOutcome } from './CaseOutcome';
import { ResultComposer } from './ResultComposer';

const CaseCard = (props: TestPartProps) => {
  const { test } = props;
  const view = useCaseHead(test);
  const { controls, toRun, head } = view;
  const { layout } = useTestsShared();
  const compact = layout === 'compact';
  const chips = (
    <Flex gap="xs" wrap>
      {view.chips.map((chip) => <Tag key={chip.id} {...chip.look} title={chip.title}>{chip.code ? <ShownCode id={chip.text} board={head.boardKey} /> : chip.text}</Tag>)}
      {compact && <CaseOutcome test={test} view={view} />}
    </Flex>
  );
  const composer = controls.open && <ResultComposer testId={test.id} outcome={controls.open} controls={controls} />;
  const card = (
    <RecordCard
      head={head}
      title={view.title || test.id}
      openLabel={`Open ${test.id}: ${view.title || test.id}`}
      onOpen={view.open}
      time={view.time}
      compact={compact}
      chevronOnIcon
      menuOnHover
      signals={chips}
      attrs={{ 'data-swap-item': '', 'data-test': test.id, ...(toRun ? { 'data-to-run': '' } : {}) }}
      onKeyDown={rowKeys(toRun, controls)}
      foot={(
        <Flex direction="column" gap="xs">
          <CaseOutcome test={test} view={view} />
          {composer}
        </Flex>
      )}
    />
  );
  if (!compact) return card;
  return (
    <Flex direction="column" gap="xs">
      {card}
      {composer}
    </Flex>
  );
};

export { CaseCard };
