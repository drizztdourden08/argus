/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { FrameCode, RecordFrame, RecordGroup, TypeIcon } from '../../../../shell';
import { ProjectLink } from '../../projects';
import type { EntriesProps } from '../boards-view.type';

const WaitingOnYou = (props: EntriesProps) => {
  const waiting = props.entries.flatMap(({ record, card }) => (record.archived ? [] : (card?.ownerItems ?? []).map((item) => ({ board: record.key, item }))));
  if (waiting.length === 0) return null;
  return (
    <RecordGroup header={<Text weight={600}>Waiting on you</Text>} count={waiting.length} label="Waiting on you">
      {waiting.map(({ board, item }) => (
        <RecordFrame
          key={`${board}/${item.id}`}
          name={item.id}
          density="row"
          icon={<TypeIcon kind={item.type} />}
          code={<FrameCode>{item.id}</FrameCode>}
          title={<Text clamp={1}>{item.title}</Text>}
          content={<ProjectLink project={board} />}
          attrs={{ role: 'listitem' }}
        >
          {item.stage && <Text variant="caption" tone="muted">{item.stage}</Text>}
        </RecordFrame>
      ))}
    </RecordGroup>
  );
};

export { WaitingOnYou };
