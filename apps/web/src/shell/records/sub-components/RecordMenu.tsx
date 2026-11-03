/* @layer renderer-app @kind component */
import { DropdownMenu } from '@drizztdourden08/tessera';
import { MENU_ICON } from '../head.constants';
import type { RecordMenuProps } from '../records.type';

const RecordMenu = (props: RecordMenuProps) => {
  const { groups, label } = props;
  if (groups.length === 0) return null;
  return (
    <DropdownMenu label={label} trigger={{ label, icon: MENU_ICON, iconOnly: true }} groups={groups} variant="ghost" size="2xs" align="end" itemSize="roomy" />
  );
};

export { RecordMenu };
