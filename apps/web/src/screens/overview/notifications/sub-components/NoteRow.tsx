/* @layer renderer-app @kind component */
import { ListItemRow, Stack, Tag, Text } from '@drizztdourden08/tessera';
import { agoText, orText } from '../../format';
import { ProjectLink } from '../../projects';
import { isFresh } from '../../seen';
import { LEVEL_LOOK, noteLevel, noteProject, toastNote } from '../model';
import type { NoteRowProps } from '../notifications.type';

const NoteRow = (props: NoteRowProps) => {
  const { note, now, freshSince } = props;
  const level = noteLevel(note);
  const project = noteProject(note);
  const toastText = toastNote(note);
  return (
    <ListItemRow
      icon={<Tag {...LEVEL_LOOK[level]}>{level}</Tag>}
      name={orText(note.title, note.text ?? '')}
      nameAction={isFresh(note.at, freshSince) ? <Tag variant="urgency" color="info">new</Tag> : undefined}
      meta={(
        <Stack gap="2xs">
          {note.title ? <Text>{note.text ?? ''}</Text> : null}
          <Text variant="caption" tone={toastText.bad ? 'danger' : 'muted'} title={note.why ?? ''}>{toastText.text}</Text>
        </Stack>
      )}
      columns={[
        { primary: <Text title={note.at ?? ''}>{agoText(note.at, now)}</Text>, align: 'end' },
        { primary: project ? <ProjectLink project={project} /> : '', secondary: note.thread ?? '' },
      ]}
    />
  );
};

export { NoteRow };
