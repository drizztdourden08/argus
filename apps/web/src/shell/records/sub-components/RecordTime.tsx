/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { Age } from '../../Age';
import type { RecordTimeProps } from '../records.type';

const RecordTime = (props: RecordTimeProps) => {
  const { time } = props;
  if (time === undefined || time === null || time === '') return null;
  return (
    <Text as="span" variant="caption" leading="16" tone="muted">
      <Age value={time} />
    </Text>
  );
};

export { RecordTime };
