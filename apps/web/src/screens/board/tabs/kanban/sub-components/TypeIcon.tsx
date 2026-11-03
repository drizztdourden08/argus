/* @layer renderer-app @kind component */
import { Icon, Tooltip } from '@drizztdourden08/tessera';
import type { IconName } from '@drizztdourden08/tessera';
import { itemTypeOf } from '../../../../../data/registry';
import { HEAD_MARK_CLASS, HEAD_SIZE_PX } from '../head.constants';
import type { TypeIconProps } from '../kanban-props.type';
import { categoryColor } from '../model';
import './HeadRow.css';

const TypeIcon = (props: TypeIconProps) => {
  const type = itemTypeOf(props.typeId);
  if (!type) return null;
  const color = categoryColor(type.color);
  return (
    <Tooltip content={type.label}>
      <span className={HEAD_MARK_CLASS} data-type-mark={type.id}>
        <Icon name={type.icon as IconName} size={HEAD_SIZE_PX} color={color ? `var(--c-tag-${color})` : 'var(--c-text-dim)'} label={type.label} />
      </span>
    </Tooltip>
  );
};

export { TypeIcon };
