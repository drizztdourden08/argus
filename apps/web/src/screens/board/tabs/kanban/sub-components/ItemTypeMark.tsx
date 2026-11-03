/* @layer renderer-app @kind component */
import type { ItemTypeMarkProps } from '../kanban-props.type';
import { typeOf } from '../model';
import { TypeIcon } from './TypeIcon';

const ItemTypeMark = (props: ItemTypeMarkProps) => <TypeIcon typeId={typeOf(props.item)} />;

export { ItemTypeMark };
