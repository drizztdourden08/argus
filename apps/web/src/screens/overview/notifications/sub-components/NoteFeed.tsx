/* @layer renderer-app @kind component */
import { EmptyState, ListItemList } from '@drizztdourden08/tessera';
import { NOTES_HEADER, NOTES_SHAPE } from '../notifications.constants';
import type { NoteFeedProps } from '../notifications.type';
import { NoteRow } from './NoteRow';

const NoteFeed = (props: NoteFeedProps) => {
  const { notes, freshSince } = props;
  if (notes.length === 0) return <EmptyState message="Nothing matches these filters." />;
  return (
    <ListItemList label="Notifications" header={NOTES_HEADER} shape={NOTES_SHAPE} count={notes.length}>
      {notes.map((note) => <NoteRow key={note.id} note={note} freshSince={freshSince} />)}
    </ListItemList>
  );
};

export { NoteFeed };
