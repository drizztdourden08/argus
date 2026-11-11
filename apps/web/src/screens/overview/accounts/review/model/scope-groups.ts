/* @layer renderer-app @kind logic */
import type { MenuGroup } from '@drizztdourden08/tessera';
import type { ScopeMenuArgs } from '../review-tree-props.type';
import { scopeChoiceItems } from './scope-choice-items';
import { scopeEditItems } from './scope-edit-items';

const scopeGroups = (args: ScopeMenuArgs): MenuGroup[] => [
  { id: 'edits', items: scopeEditItems(args) },
  { id: 'choices', items: scopeChoiceItems(args) },
];

export { scopeGroups };
