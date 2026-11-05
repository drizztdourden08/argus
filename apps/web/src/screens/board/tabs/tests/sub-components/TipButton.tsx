/* @layer renderer-app @kind component */
import { Icon, IconButton, Tooltip } from '@drizztdourden08/tessera';
import type { TipButtonProps } from '../tests-head-props.type';

const TipButton = (props: TipButtonProps) => {
  const { icon, tip, ...button } = props;
  return (
    <Tooltip content={tip} focusable>
      <IconButton size="xs" {...button}>
        <Icon name={icon} />
      </IconButton>
    </Tooltip>
  );
};

export { TipButton };
