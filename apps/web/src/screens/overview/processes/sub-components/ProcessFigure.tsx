/* @layer renderer-app @kind component */
import { Inline, Text } from '@drizztdourden08/tessera';
import type { ProcessFigureProps } from '../processes.type';

const ProcessFigure = (props: ProcessFigureProps) => {
  const { before, value, after, title } = props;
  return (
    <Inline gap="xs" align="baseline" title={title}>
      {before ? <Text variant="subtitle">{before}</Text> : null}
      <Text weight={600} numeric>{value}</Text>
      {after ? <Text variant="subtitle">{after}</Text> : null}
    </Inline>
  );
};

export { ProcessFigure };
