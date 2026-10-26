/* @layer renderer-app @kind component */
import { Tag } from '@drizztdourden08/tessera';
import type { QuietBadgeProps } from '../kanban-props.type';

const QuietBadge = (props: QuietBadgeProps) => {
  const { count } = props;
  return (
    <Tag
      variant="urgency"
      color="warning"
      title={`${count} ${count === 1 ? 'item has' : 'items have'} had no change for too long: their thread must start, update or close them.`}
    >
      {`${count} quiet`}
    </Tag>
  );
};

export { QuietBadge };
