/* @layer renderer-app @kind component */
import { EmptyState } from '@drizztdourden08/tessera';
import { FeedTable } from '../../sub-components/FeedTable';
import type { NoteFeedProps } from '../notifications.type';
import { NoteRow } from './NoteRow';

const NoteFeed = (props: NoteFeedProps) => {
  const { notes, freshSince } = props;
  if (notes.length === 0) return <EmptyState message="Nothing matches these filters." />;
  return (
    <FeedTable label="Notifications">
      {notes.map((note) => <NoteRow key={note.id} note={note} freshSince={freshSince} />)}
    </FeedTable>
  );
};

export { NoteFeed };
