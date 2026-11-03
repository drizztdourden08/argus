/* @layer renderer-app @kind component */
import { TypeMark } from '@drizztdourden08/tessera';
import type { IconName } from '@drizztdourden08/tessera';
import { itemTypeOf } from '../../../../../data/registry';
import type { ItemTypeMarkProps } from '../kanban-props.type';
import { categoryColor, typeOf } from '../model';

const ItemTypeMark = (props: ItemTypeMarkProps) => {
  const type = itemTypeOf(typeOf(props.item));
  if (!type) return null;
  return <TypeMark icon={type.icon as IconName} label={type.label} color={categoryColor(type.color)} size="xs" />;
};

export { ItemTypeMark };
