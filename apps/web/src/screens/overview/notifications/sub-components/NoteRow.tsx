/* @layer renderer-app @kind component */
import { ListItemRow, Stack, Tag, Text } from '@drizztdourden08/tessera';
import { orText } from '../../../../format';
import { ProjectLink } from '../../projects';
import { isFresh } from '../../seen';
import { ShortAge } from '../../sub-components/ShortAge';
import { LEVEL_LOOK, noteLevel, noteProject, toastNote } from '../model';
import type { NoteRowProps } from '../notifications.type';

const NoteRow = (props: NoteRowProps) => {
  const { note, freshSince } = props;
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
        { primary: note.at ? <ShortAge value={note.at} ago /> : '', align: 'end' },
        { primary: project ? <ProjectLink project={project} /> : '', secondary: note.thread ?? '' },
      ]}
    />
  );
};

export { NoteRow };
