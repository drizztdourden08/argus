/* @layer renderer-app @kind component */
import { DropdownMenu } from '@drizztdourden08/tessera';
import { testMenuGroups } from '../behavior/test-menu-groups';
import { useTestActions } from '../behavior/useTestActions';
import { useTestsShared } from '../behavior/useTestsShared';
import type { TestPartProps } from '../tests-props.type';

const TestMenu = (props: TestPartProps) => {
  const { test } = props;
  const shared = useTestsShared();
  const actions = useTestActions(shared.boardKey);
  const label = `More actions for ${test.id}`;
  return (
    <DropdownMenu
      label={label}
      trigger={{ label, icon: 'ellipsis', iconOnly: true }}
      groups={testMenuGroups({ test, shared, rerun: () => void actions.rerun(test.id) })}
      variant="ghost"
      size="xs"
      align="end"
    />
  );
};

export { TestMenu };
