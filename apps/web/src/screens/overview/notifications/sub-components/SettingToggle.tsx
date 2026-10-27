/* @layer renderer-app @kind component */
import { Tag, Tooltip } from '@drizztdourden08/tessera';
import { CHIP_LOOK } from '../../overview.constants';
import { settingTip } from '../model';
import { PART_LABEL } from '../notifications.constants';
import type { SettingToggleProps } from '../notifications.type';

const SettingToggle = (props: SettingToggleProps) => {
  const { part, level, on, busy, onToggle } = props;
  return (
    <Tooltip content={settingTip(part, level, on)}>
      <Tag {...CHIP_LOOK} selected={on} disabled={busy} onSelect={onToggle}>{PART_LABEL[part]}</Tag>
    </Tooltip>
  );
};

export { SettingToggle };
