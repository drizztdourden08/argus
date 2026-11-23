/* @layer renderer-app @kind component */
import { settingChips } from '../behavior/setting-chips';
import type { SettingChipsProps } from '../behavior/chat-settings.type';
import { SettingChip } from './SettingChip';

const SettingChips = (props: SettingChipsProps) => {
  const { chat } = props;
  if (!chat.settings) return null;
  return (
    <>
      {settingChips(chat.settings).map((chip) => (
        <SettingChip
          key={chip.key}
          label={chip.label}
          current={chip.current}
          selected={chip.selected}
          choices={chip.choices}
          busy={chat.busy}
          onPick={(id) => chat.pick(chip.key, id)}
        />
      ))}
    </>
  );
};

export { SettingChips };
