/* @layer renderer-app @kind logic */
import { ownEntry } from './own-entry';
import { ITEM_REGISTRY } from './registry.constants';

const knowsLead = (key: string): boolean => ownEntry(ITEM_REGISTRY.people.leads, key) !== undefined;

export { knowsLead };
