/* @layer renderer-app @kind component */
import { Flex, Icon, IconButton, Pressable, Text } from '@drizztdourden08/tessera';
import { StatusTag } from '../sub-components/StatusTag';
import type { CaseRowProps } from './pane.type';

const CaseRow = (props: CaseRowProps) => {
  const { id, data, onRemove } = props;
  const test = data.allTests.find((candidate) => candidate.id === id);
  return (
    <Flex gap="sm" align="center" wrap role="listitem">
      <Pressable aria-label={`Open ${id}`} onClick={() => data.open(id)}>
        <Text as="span" variant="caption" weight={600} mono>{id}</Text>
      </Pressable>
      {test ? <Text as="span" smartTimes>{test.title}</Text> : <Text as="span" tone="muted">not on this board</Text>}
      {test && <StatusTag status={test.status} />}
      <IconButton label={`Remove ${id}`} size="sm" variant="ghost" onClick={() => onRemove(id)}><Icon name="x" /></IconButton>
    </Flex>
  );
};

export { CaseRow };
