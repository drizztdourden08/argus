/* @layer renderer-app @kind component */
import { Tag, Text } from '@drizztdourden08/tessera';
import { useProjectName } from '../../projects';
import { Age, FrameCode, RecordFrame } from '../../../../shell';
import { GOOD_LOOK, INFO_LOOK, WARN_LOOK } from '../boards-run.constants';
import { KIND_WORD } from '../boards-send.constants';
import type { SentRowProps } from '../boards.type';
import { sentTarget } from '../model';

const SentRow = (props: SentRowProps) => {
  const { entry } = props;
  const nameOf = useProjectName();
  const kind = entry.kind ?? '';
  const target = sentTarget(entry.project, nameOf);
  const from = entry.author === 'human' ? 'you' : 'Overlord';
  return (
    <RecordFrame
      name={entry.id}
      density="row"
      icon={<Tag {...(kind === 'decision' ? INFO_LOOK : GOOD_LOOK)}>{KIND_WORD[kind] ?? kind}</Tag>}
      code={<FrameCode>{entry.id}</FrameCode>}
      title={<Text clamp={1}>{entry.text ?? ''}</Text>}
      content={
        <>
          <Tag {...(entry.status === 'delivered' ? GOOD_LOOK : WARN_LOOK)}>{entry.status ?? 'sent'}</Tag>
          <Text>{from}</Text>
          {entry.createdAt && <Age value={entry.createdAt} />}
        </>
      }
      attrs={{ role: 'listitem' }}
    >
      <Text variant="caption" tone="muted">{`to ${target}`}</Text>
    </RecordFrame>
  );
};

export { SentRow };
