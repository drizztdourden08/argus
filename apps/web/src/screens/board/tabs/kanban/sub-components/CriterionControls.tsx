/* @layer renderer-app @kind component */
import { Button, Flex } from '@drizztdourden08/tessera';
import type { CriterionControlsProps } from '../kanban-props.type';

const CriterionControls = (props: CriterionControlsProps) => {
  const { item, index, ops } = props;
  const criteria = item.ac ?? [];
  const n = index + 1;
  const shift = (to: number) => void ops.criterion(item, { do: 'move', n, to }, `Reordered the criteria of ${item.id}.`);
  const remove = () => void ops.criterion(item, { do: 'rm', n, prev: criteria[index]?.text ?? '' }, `Criterion ${n} of ${item.id} removed.`);
  return (
    <Flex gap="xs">
      <Button size="xs" variant="ghost" aria-label={`Move criterion ${n} up`} disabled={index === 0} onClick={() => shift(n - 1)}>Up</Button>
      <Button size="xs" variant="ghost" aria-label={`Move criterion ${n} down`} disabled={index === criteria.length - 1} onClick={() => shift(n + 1)}>Down</Button>
      <Button size="xs" variant="ghost" aria-label={`Remove criterion ${n}`} onClick={remove}>Remove</Button>
    </Flex>
  );
};

export { CriterionControls };
