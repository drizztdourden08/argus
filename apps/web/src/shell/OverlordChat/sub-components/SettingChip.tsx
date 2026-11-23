/* @layer renderer-app @kind component */
import { DropdownMenu } from '@drizztdourden08/tessera';
import type { MenuGroup } from '@drizztdourden08/tessera';
import type { SettingChipProps } from '../OverlordChat.type';

const SettingChip = (props: SettingChipProps) => {
  const { label, current, selected, choices, busy, onPick } = props;
  const groups: MenuGroup[] = [
    {
      id: 'choices',
      items: choices.map((choice) => ({
        id: choice.id,
        label: choice.label,
        kind: 'radio',
        checked: choice.id === selected,
        onSelect: () => onPick(choice.id),
      })),
    },
  ];
  return <DropdownMenu trigger={{ label: current }} label={label} variant="ghost" size="xs" disabled={busy} groups={groups} />;
};

export { SettingChip };
