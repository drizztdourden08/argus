/* @layer renderer-app @kind component */
import { Checkbox, List, ListItem } from '@drizztdourden08/tessera';
import type { CriteriaListProps } from '../WorkItems.type';

const readOnly = (): void => undefined;

const CriteriaList = (props: CriteriaListProps) => {
  const { criteria, label } = props;
  return (
    <List aria-label={label}>
      {criteria.map((criterion) => (
        <ListItem key={criterion.n} bare>
          <Checkbox checked={criterion.done} onChange={readOnly} label={criterion.text} disabled />
        </ListItem>
      ))}
    </List>
  );
};

export { CriteriaList };
