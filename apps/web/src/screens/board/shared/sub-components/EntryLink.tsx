/* @layer renderer-app @kind component */
import { Pressable, Text } from '@drizztdourden08/tessera';
import { useOpenEntry } from '../behavior/useOpenEntry';
import type { EntryLinkProps } from '../shared.type';

const EntryLink = (props: EntryLinkProps) => {
  const { id } = props;
  const open = useOpenEntry();
  return (
    <Pressable aria-label={`Open ${id}`} onClick={() => open(id)}>
      <Text as="span" variant="caption" tone="muted" mono>{id}</Text>
    </Pressable>
  );
};

export { EntryLink };
