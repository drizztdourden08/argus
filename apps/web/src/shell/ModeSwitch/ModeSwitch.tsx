/* @layer renderer-app @kind component */
import { SegmentedControl } from '@drizztdourden08/tessera';
import type { ShellMode } from '../AppShell/AppShell.type';
import type { ModeSwitchProps } from './ModeSwitch.type';

const ModeSwitch = (props: ModeSwitchProps) => {
  const { mode, boardsAvailable, onChange } = props;
  const options = [
    { value: 'overview' as const, label: 'Overview' },
    { value: 'boards' as const, label: 'Boards', disabled: boardsAvailable ? false : 'No boards yet' },
  ];
  return <SegmentedControl<ShellMode> aria-label="Mode" size="sm" selectedTone="neutral" options={options} value={mode} onChange={onChange} />;
};

export { ModeSwitch };
