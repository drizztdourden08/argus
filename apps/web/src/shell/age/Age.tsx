/* @layer renderer-app @kind component */
import { Time, Tooltip, formatDate } from '@drizztdourden08/tessera';
import { formatAgo, toMillis } from '../../format';
import { useAgeClock } from './behavior/useAgeClock';
import type { AgeProps } from './Age.type';
import './Age.css';

const Age = (props: AgeProps) => {
  const { value, ago = false } = props;
  const now = useAgeClock();
  const at = toMillis(value);
  const time = (
    <Time className="age" dateTime={Number.isFinite(at) ? new Date(at).toISOString() : undefined}>
      {formatAgo(value, now)}
      {ago ? ' ago' : null}
    </Time>
  );
  return Number.isFinite(at) ? <Tooltip content={formatDate(at, 'datetime')}>{time}</Tooltip> : time;
};

export { Age };
