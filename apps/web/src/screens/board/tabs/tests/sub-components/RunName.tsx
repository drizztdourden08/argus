/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { ShownCode } from '../../../../../shell';
import type { RunNameProps } from '../tests-props.type';

const RunName = (props: RunNameProps) => {
  const { id, title, muted = false } = props;
  return (
    <>
      <Text font="title" tone={muted ? 'muted' : undefined} className="run-card__tag"><ShownCode id={id} /></Text>
      {title && <Text smartTimes tone={muted ? 'muted' : 'dim'} className="run-card__title">{title}</Text>}
    </>
  );
};

export { RunName };
