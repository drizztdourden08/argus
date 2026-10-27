/* @layer renderer-app @kind component */
import { Stack, Tag, Text } from '@drizztdourden08/tessera';
import { ProjectLink } from '../../projects';
import { isFresh } from '../../seen';
import { FeedRow } from '../../sub-components/FeedRow';
import { LEVEL_LOOK, NOTE_ICON, NOTE_TONE, noteLevel, noteProject, toastNote } from '../model';
import type { NoteRowProps } from '../notifications.type';

const NoteRow = (props: NoteRowProps) => {
  const { note, freshSince } = props;
  const level = noteLevel(note);
  const project = noteProject(note);
  const toastText = toastNote(note);
  return (
    <FeedRow icon={NOTE_ICON[level]} tone={NOTE_TONE[level]} title={level} at={note.at}>
      <Stack gap="2xs">
        <Text>
          <Tag {...LEVEL_LOOK[level]} pill>{level}</Tag>
          {note.title ? <Text weight={600}>{` ${note.title}`}</Text> : null}
          <Text tone="dim">{` ${note.text ?? ''} `}</Text>
          {project ? <ProjectLink project={project} chip /> : null}
          {note.thread ? <Text variant="caption">{` ${project ? '/ ' : ''}${note.thread}`}</Text> : null}
          {isFresh(note.at, freshSince) ? <> <Tag variant="urgency" color="info" pill>new</Tag></> : null}
        </Text>
        <Text variant="caption" tone={toastText.bad ? 'danger' : 'muted'} title={note.why ?? ''}>{toastText.text}</Text>
      </Stack>
    </FeedRow>
  );
};

export { NoteRow };
