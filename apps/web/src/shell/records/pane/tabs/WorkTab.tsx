/* @layer renderer-app @kind component */
import { itemBody } from '../item-body';
import { placeholderBody } from '../placeholder-body';
import { ServedTab } from './ServedTab';

const WorkTab = itemBody(ServedTab, placeholderBody('work', 'Work'));

export { WorkTab };
