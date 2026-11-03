/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { RecordLink } from '../../../../../shell';
import { EDGE_WORDS } from './model/pane-model.constants';
import type { CauseRowProps } from './pane.type';

const CauseRow = (props: CauseRowProps) => {
  const { kind, id, label } = props;
  return (
    <Flex gap="xs" align="center" wrap>
      <Text variant="caption" tone="muted">{kind === 'blocks' ? 'blocks' : (EDGE_WORDS[kind] ?? kind)}</Text>
      <RecordLink id={id} />
      <Text smartTimes>{label}</Text>
    </Flex>
  );
};

export { CauseRow };
