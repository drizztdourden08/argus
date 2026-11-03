/* @layer renderer-app @kind component */
import type { ItemTypeMarkProps } from '../records.type';
import { ITEM_FALLBACK_KIND } from '../behavior/record-grammar.constants';
import { TypeIcon } from './TypeIcon';

const ItemTypeMark = (props: ItemTypeMarkProps) => <TypeIcon kind={props.item.type || ITEM_FALLBACK_KIND} size={props.size} />;

export { ItemTypeMark };
