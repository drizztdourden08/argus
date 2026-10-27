/* @layer renderer-app @kind component */
import { Flex, ProgressBar, Text } from '@drizztdourden08/tessera';
import type { CriteriaBarProps } from '../kanban-props.type';
import './CriteriaBar.css';

const CriteriaBar = (props: CriteriaBarProps) => {
  const criteria = props.item.ac ?? [];
  if (criteria.length === 0) return null;
  const met = criteria.filter((criterion) => criterion.done).length;
  return (
    <Flex gap="xs" align="center">
      <ProgressBar value={met} max={criteria.length} tone="success" className="criteria-bar" label={`${met} of ${criteria.length} criteria met`} />
      <Text variant="caption" tone="muted" numeric>{`${met}/${criteria.length}`}</Text>
    </Flex>
  );
};

export { CriteriaBar };
