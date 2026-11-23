/* @layer renderer-app @kind component */
import { Tag } from '@drizztdourden08/tessera';
import { limitTone } from '../model';
import type { LimitBadgeProps } from '../kanban-props.type';

const LimitBadge = (props: LimitBadgeProps) => {
  const { count, limit } = props;
  return (
    <Tag variant="urgency" color={limitTone(count, limit)} title={`This column holds ${count} of at most ${limit.max} (${limit.mode}).`}>
      {`${count}/${limit.max}`}
    </Tag>
  );
};

export { LimitBadge };
