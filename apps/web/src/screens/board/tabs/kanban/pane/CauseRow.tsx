/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { EntryLink } from '../../../shared';
import { RefTags } from './fields/RefTags';
import { EDGE_WORDS, ENTRY_CAUSES } from './model/pane-model.constants';
import type { CauseRowProps } from './pane.type';

const CauseRow = (props: CauseRowProps) => {
  const { kind, id, label, open } = props;
  return (
    <Flex gap="xs" align="center" wrap>
      <Text variant="caption" tone="muted">{kind === 'blocks' ? 'blocks' : (EDGE_WORDS[kind] ?? kind)}</Text>
      {ENTRY_CAUSES.includes(kind) ? <EntryLink id={id} /> : <RefTags ids={[id]} open={open} />}
      <Text smartTimes>{label}</Text>
    </Flex>
  );
};

export { CauseRow };
