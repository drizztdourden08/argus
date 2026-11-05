/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import type { RegisteredLabelProps } from '../RegisteredStrip.type';

const RegisteredLabel = (props: RegisteredLabelProps) => {
  const { rows } = props;
  const waking = rows.filter((row) => row.state === 'waking').length;
  const faults = rows.filter((row) => row.state === 'fault').length;
  return (
    <Text variant="caption" weight={600}>
      {`${rows.length} registered`}
      {waking > 0 && <> · <Text variant="caption" tone="warning" weight={600}>{`${waking} waking`}</Text></>}
      {faults > 0 && <> · <Text variant="caption" tone="danger" weight={600}>{`${faults} fault${faults > 1 ? 's' : ''}`}</Text></>}
    </Text>
  );
};

export { RegisteredLabel };
