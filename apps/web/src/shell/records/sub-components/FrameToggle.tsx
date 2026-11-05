/* @layer renderer-app @kind component */
import { Icon, IconButton } from '@drizztdourden08/tessera';
import { CHEVRON_PX } from '../head.constants';
import type { FrameToggleProps } from '../record-frame.type';

const FrameToggle = (props: FrameToggleProps) => {
  const { open, name, onToggle } = props;
  return (
    <IconButton label={`${open ? 'Collapse' : 'Expand'} ${name}`} variant="ghost" size="2xs" aria-expanded={open} onClick={onToggle}>
      <Icon name="chevron-right" size={CHEVRON_PX} className={open ? 'record-frame__chevron record-frame__chevron--open' : 'record-frame__chevron'} />
    </IconButton>
  );
};

export { FrameToggle };
