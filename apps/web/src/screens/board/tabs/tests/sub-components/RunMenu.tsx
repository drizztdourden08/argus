/* @layer renderer-app @kind component */
import { DropdownMenu } from '@drizztdourden08/tessera';
import { itemCode } from '../../../../../shell';
import { useRunMenu } from '../behavior/useRunMenu';
import type { RunActionsProps } from '../tests-head-props.type';

const RunMenu = (props: RunActionsProps) => {
  const { view, runs } = props;
  const groups = useRunMenu(view, runs);
  const label = `More actions for ${itemCode(view.item)}`;
  return <DropdownMenu label={label} trigger={{ label, icon: 'ellipsis-vertical', iconOnly: true }} groups={groups} variant="ghost" size="2xs" align="end" itemSize="roomy" />;
};

export { RunMenu };
