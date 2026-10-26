/* @layer renderer-app @kind logic */
import type { NotifyLevel } from '../../../../data/types';
import type { SettingPart } from './model.type';

const toastTip = (level: NotifyLevel, on: boolean): string => {
  const tip = `${on ? 'A toast is shown for' : 'No toast for'} ${level}`;
  return level === 'info' ? `${tip} (info is recorded only by default)` : tip;
};

const soundTip = (level: NotifyLevel, on: boolean): string =>
  on ? `The ${level} toast plays the system notification sound` : `The ${level} toast is silent`;

const settingTip = (part: SettingPart, level: NotifyLevel, on: boolean): string =>
  part === 'toast' ? toastTip(level, on) : soundTip(level, on);

export { settingTip };
