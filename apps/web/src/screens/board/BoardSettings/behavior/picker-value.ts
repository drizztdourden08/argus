/* @layer renderer-app @kind logic */
import type { IconPickerValue } from '@drizztdourden08/tessera';
import { resolveBoardIcon } from '../../../../board-icon';

const pickerValue = (setting: string | null): IconPickerValue => {
  const { source } = resolveBoardIcon('', setting);
  return { kind: 'icon', name: source.kind === 'icon' ? source.name : 'gamepad-2' };
};

export { pickerValue };
