/* @layer renderer-app @kind logic */
import { ITEM_REGISTRY } from '../../../../../data/registry';
import type { Option } from '../../../shared';

const typeChoices = (): Option[] => ITEM_REGISTRY.types.filter((type) => type.collection === 'items').map((type) => ({ value: type.id, label: type.label }));

export { typeChoices };
