/* @layer renderer-app @kind component */
import { Tag } from '@drizztdourden08/tessera';
import type { StatusTagProps } from '../drawer-props.type';
import { STATUS_TONES } from '../kanban.constants';

const StatusTag = (props: StatusTagProps) => {
  const { status } = props;
  const color = STATUS_TONES[status];
  return color ? <Tag variant="urgency" color={color}>{status}</Tag> : <Tag>{status}</Tag>;
};

export { StatusTag };
