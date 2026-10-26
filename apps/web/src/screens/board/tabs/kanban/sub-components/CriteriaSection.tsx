/* @layer renderer-app @kind component */
import { Box, Text } from '@drizztdourden08/tessera';
import { useCriteriaReorder } from '../behavior/useCriteriaReorder';
import type { CriteriaSectionProps } from '../drawer-props.type';
import { LIMITS, textProblem } from '../model';
import { AddLine } from './AddLine';
import { CriterionRow } from './CriterionRow';
import { DrawerSection } from './DrawerSection';
import './CriteriaSection.css';

const CriteriaSection = (props: CriteriaSectionProps) => {
  const { item, tests, ops } = props;
  const criteria = item.ac ?? [];
  const reorder = useCriteriaReorder(item, ops);
  const met = criteria.filter((criterion) => criterion.done).length;
  const provedBy = (n: number): string[] => tests.filter((test) => test.ac.includes(n)).map((test) => test.id);
  return (
    <DrawerSection title={`Acceptance criteria, ${met}/${criteria.length} met`}>
      {criteria.length === 0 && <Text tone="muted">None entered.</Text>}
      <Box ref={reorder.listRef} className="criteria-list" role="group" aria-label="Acceptance criteria">
        {criteria.map((criterion, index) => (
          <CriterionRow
            key={reorder.keys[index]}
            item={item}
            index={index}
            rowKey={reorder.keys[index] ?? ''}
            proving={provedBy(index + 1)}
            ops={ops}
            reorder={reorder}
          />
        ))}
      </Box>
      <Text variant="caption" tone="muted" aria-live="polite">{reorder.message}</Text>
      <AddLine
        label="Add a criterion"
        buttonLabel="Add criterion"
        limit={LIMITS.criterion}
        problemOf={(text) => textProblem('criterion', text)}
        onAdd={(text) => ops.criterion(item, { do: 'add', text }, `Criterion added to ${item.id}.`)}
      />
    </DrawerSection>
  );
};

export { CriteriaSection };
