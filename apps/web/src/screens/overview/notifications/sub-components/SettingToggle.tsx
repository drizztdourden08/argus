/* @layer renderer-app @kind component */
import { Toggle, Tooltip } from '@drizztdourden08/tessera';
import { settingTip } from '../model';
import { PART_LABEL } from '../notifications.constants';
import type { SettingToggleProps } from '../notifications.type';

const SettingToggle = (props: SettingToggleProps) => {
  const { part, level, on, busy, onToggle } = props;
  return (
    <Tooltip content={settingTip(part, level, on)}>
      <Toggle label={PART_LABEL[part]} checked={on} disabled={busy} onChange={onToggle} />
    </Tooltip>
  );
};

export { SettingToggle };
