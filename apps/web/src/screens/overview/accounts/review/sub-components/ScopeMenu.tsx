/* @layer renderer-app @kind component */
import { DropdownMenu } from '@drizztdourden08/tessera';
import { scopeGroups } from '../model';
import type { ScopeMenuProps } from '../review-tree-props.type';
import { RowTip } from './RowTip';

const ScopeMenu = (props: ScopeMenuProps) => {
  const { label, ...args } = props;
  return (
    <RowTip label={label}>
      <DropdownMenu variant="ghost" size="xs" trigger={{ label, icon: 'layers', iconOnly: true }} groups={scopeGroups(args)} />
    </RowTip>
  );
};

export { ScopeMenu };
