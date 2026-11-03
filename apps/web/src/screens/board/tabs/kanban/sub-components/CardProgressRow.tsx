/* @layer renderer-app @kind component */
import { Flex, ProgressBar, Text } from '@drizztdourden08/tessera';
import type { CardProgressRowProps } from '../kanban-props.type';
import './CardProgressRow.css';

const CardProgressRow = (props: CardProgressRowProps) => {
  const [first, ...rest] = props.segments;
  if (!first) return null;
  return (
    <Flex gap="xs" align="center">
      <ProgressBar value={first.met} max={Math.max(first.total, 1)} tone="success" className="card-progress__bar" label={first.title} />
      <Text variant="caption" tone="muted" numeric title={first.title}>{first.text}</Text>
      {rest.map((segment) => <Text key={segment.title} variant="caption" tone="muted" numeric title={segment.title}>{segment.text}</Text>)}
    </Flex>
  );
};

export { CardProgressRow };
