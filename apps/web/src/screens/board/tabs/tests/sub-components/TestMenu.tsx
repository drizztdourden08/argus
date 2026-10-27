/* @layer renderer-app @kind component */
import { ContextMenu, DropdownMenu } from '@drizztdourden08/tessera';
import { testMenuGroups } from '../behavior/test-menu-groups';
import { useTestActions } from '../behavior/useTestActions';
import { useTestsShared } from '../behavior/useTestsShared';
import type { TestMenuProps } from '../tests-props.type';

const TestMenu = (props: TestMenuProps) => {
  const { test, menu } = props;
  const shared = useTestsShared();
  const actions = useTestActions(shared.boardKey);
  const label = `More actions for ${test.id}`;
  const groups = testMenuGroups({ test, shared, rerun: () => void actions.rerun(test.id) });
  return (
    <>
      <DropdownMenu label={label} trigger={{ label, icon: 'ellipsis-vertical', iconOnly: true }} groups={groups} variant="ghost" size="xs" align="end" />
      <ContextMenu open={menu.at !== null} point={menu.at ?? undefined} label={label} groups={groups} variant="ghost" onClose={menu.close} />
    </>
  );
};

export { TestMenu };
