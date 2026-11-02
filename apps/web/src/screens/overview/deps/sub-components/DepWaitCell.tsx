/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { isLive } from '../../../../data/deps';
import { Age } from '../../../../shell';
import type { DepRowProps } from '../deps.type';
import { startedAt, waitText } from '../model';

const DepWaitCell = (props: DepRowProps) => {
  const { doc, now } = props;
  return (
    <Text size="md" tone="dim" numeric title={doc.title}>
      {isLive(doc.state) ? <Age value={startedAt(doc)} /> : waitText(doc, now)}
    </Text>
  );
};

export { DepWaitCell };
