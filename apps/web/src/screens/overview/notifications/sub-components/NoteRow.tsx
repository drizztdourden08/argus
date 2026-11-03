/* @layer renderer-app @kind component */
import { Flex, Tag, Text } from '@drizztdourden08/tessera';
import { Age } from '../../../../shell';
import { ProjectLink } from '../../projects';
import { OverviewRow, OWNER_ASSIGNEE, recordOfText } from '../../records';
import { isFresh } from '../../seen';
import { LEVEL_LOOK, noteLevel, noteProject, toastNote } from '../model';
import type { NoteRowProps } from '../notifications.type';

const NoteRow = (props: NoteRowProps) => {
  const { note, freshSince } = props;
  const level = noteLevel(note);
  const project = noteProject(note);
  const toastText = toastNote(note);
  const title = note.title || note.text || '';
  const detail = note.title ? note.text ?? '' : '';
  return (
    <OverviewRow
      target={recordOfText(`${title} ${detail}`, project, level)}
      assignee={OWNER_ASSIGNEE}
      title={title}
      chips={[{ id: 'toast', text: toastText.text, title: note.why ?? '' }]}
      trailing={
        <Flex gap="xs" align="center">
          {detail ? <Text variant="caption" tone="dim" clamp={1}>{detail}</Text> : null}
          <Tag {...LEVEL_LOOK[level]} pill>{level}</Tag>
          {project ? <ProjectLink project={project} chip /> : null}
          {note.thread ? <Text variant="caption">{`/ ${note.thread}`}</Text> : null}
          {isFresh(note.at, freshSince) ? <Tag variant="urgency" color="info" pill>new</Tag> : null}
          {note.at ? <Text variant="caption" tone="muted"><Age value={note.at} ago /></Text> : null}
        </Flex>
      }
    />
  );
};

export { NoteRow };
