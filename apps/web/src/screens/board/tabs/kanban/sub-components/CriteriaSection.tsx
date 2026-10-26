/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import type { CriteriaSectionProps } from '../kanban-props.type';
import { CriterionAdd } from './CriterionAdd';
import { CriterionRow } from './CriterionRow';
import { DrawerSection } from './DrawerSection';

const CriteriaSection = (props: CriteriaSectionProps) => {
  const { item, tests, ops } = props;
  const criteria = item.ac ?? [];
  const met = criteria.filter((criterion) => criterion.done).length;
  const provedBy = (n: number): string[] => tests.filter((test) => test.ac.includes(n)).map((test) => test.id);
  return (
    <DrawerSection title={`Acceptance criteria, ${met}/${criteria.length} met`}>
      <Flex direction="column" gap="sm" role="group" aria-label="Acceptance criteria">
        {criteria.length === 0 && <Text tone="muted">None entered.</Text>}
        {criteria.map((criterion, index) => (
          <CriterionRow key={`${index}-${criterion.text}`} item={item} index={index} proving={provedBy(index + 1)} ops={ops} />
        ))}
      </Flex>
      <CriterionAdd item={item} ops={ops} />
    </DrawerSection>
  );
};

export { CriteriaSection };
