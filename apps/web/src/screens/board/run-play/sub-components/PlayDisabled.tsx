/* @layer renderer-app @kind component */
import { Button, Tooltip } from '@drizztdourden08/tessera';
import type { PlayDisabledProps } from '../run-play.type';

const PlayDisabled = (props: PlayDisabledProps) => {
  const { label, hint } = props;
  return (
    <Tooltip content={hint} focusable>
      <Button size="sm" variant="primary" disabled aria-label={`Play ${label}`} title={hint}>Play</Button>
    </Tooltip>
  );
};

export { PlayDisabled };
