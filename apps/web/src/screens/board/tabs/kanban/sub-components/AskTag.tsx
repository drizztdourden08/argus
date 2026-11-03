/* @layer renderer-app @kind component */
import { Pressable, Tag } from '@drizztdourden08/tessera';
import { useOpenEntry } from '../../../shared';
import type { AskTagProps } from '../kanban-props.type';
import { askTag } from '../model';

const AskTag = (props: AskTagProps) => {
  const { link } = props;
  const open = useOpenEntry();
  const spec = askTag(link);
  return (
    <Pressable aria-label={`Open ${link.id}`} onClick={() => open(link.id)}>
      <Tag variant="urgency" color={spec.color} icon={spec.icon} title={spec.title}>{spec.label}</Tag>
    </Pressable>
  );
};

export { AskTag };
