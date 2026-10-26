/* @layer renderer-app @kind component */
import { Button, Tooltip } from '@drizztdourden08/tessera';
import type { PlayDisabledProps } from '../tests-props.type';

const PlayDisabled = (props: PlayDisabledProps) => {
  const { tag, hint } = props;
  return (
    <Tooltip content={hint} focusable>
      <Button size="sm" variant="primary" disabled aria-label={`Play ${tag}`} title={hint}>Play</Button>
    </Tooltip>
  );
};

export { PlayDisabled };
