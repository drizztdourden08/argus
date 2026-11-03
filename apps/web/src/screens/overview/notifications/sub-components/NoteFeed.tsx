/* @layer renderer-app @kind component */
import { EmptyState } from '@drizztdourden08/tessera';
import { OverviewList } from '../../records';
import type { NoteFeedProps } from '../notifications.type';
import { NoteRow } from './NoteRow';

const NoteFeed = (props: NoteFeedProps) => {
  const { notes, freshSince } = props;
  if (notes.length === 0) return <EmptyState message="Nothing matches these filters." />;
  return (
    <OverviewList label="Notifications">
      {notes.map((note) => <NoteRow key={note.id} note={note} freshSince={freshSince} />)}
    </OverviewList>
  );
};

export { NoteFeed };
