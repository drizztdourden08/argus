/* @layer renderer-app @kind component */
import { Button, Flex, SegmentedControl } from '@drizztdourden08/tessera';
import { OwnerBox } from '../../OwnerBox';
import { useItemNote } from '../behavior/useItemNote';
import { NOTE_OPTIONS } from '../WorkItems.constants';
import type { ItemNoteFormProps } from '../WorkItems.type';

const ItemNoteForm = (props: ItemNoteFormProps) => {
  const { item, poster } = props;
  const note = useItemNote(item, poster);
  const decision = note.kind === 'decision';
  return (
    <Flex direction="column" gap="xs">
      <SegmentedControl label={`Kind of post on ${item.id}`} value={note.kind} options={NOTE_OPTIONS} onChange={note.setKind} size="sm" />
      <OwnerBox
        label={decision ? `The decision for ${item.thread}.` : `A note for ${item.thread}.`}
        value={note.text}
        onChange={note.setText}
        onSubmit={note.send}
        disabled={note.busy}
      />
      <Flex>
        <Button size="sm" disabled={note.busy} onClick={note.send}>{decision ? 'Post decision' : 'Post note'}</Button>
      </Flex>
    </Flex>
  );
};

export { ItemNoteForm };
