/* @layer renderer-app @kind component */
import { Pressable, Tag, Text } from '@drizztdourden08/tessera';
import { Age, CALM_LOOK, FrameCode, RecordFrame } from '../../../../shell';
import { BoardIcon } from '../../../../shell/BoardIcon';
import { useRecordPane } from '../../../../shell/records';
import { WARN_LOOK } from '../boards-run.constants';
import type { BoardRowProps } from '../boards-view.type';

const BoardRow = (props: BoardRowProps) => {
  const { entry } = props;
  const { record, updatedAt } = entry;
  const pane = useRecordPane();
  const needs = record.counts.needsYou;
  return (
    <RecordFrame
      name={record.id}
      density="row"
      icon={<BoardIcon boardKey={record.key} icon={record.icon} size="sm" />}
      code={<FrameCode>{record.id}</FrameCode>}
      title={
        <Pressable onClick={() => pane.open(record.id)} aria-label={`Open ${record.id}: ${record.name}`}>
          <Text clamp={1}>{record.name}</Text>
        </Pressable>
      }
      content={
        <>
          <Tag {...CALM_LOOK}>{record.archived ? 'archived' : record.lead}</Tag>
          {needs > 0 && <Tag {...WARN_LOOK}>{`${needs} need you`}</Tag>}
          {updatedAt && <Age value={updatedAt} />}
        </>
      }
      attrs={{ role: 'listitem' }}
    >
      {record.goal && <Text variant="caption" tone="muted">{record.goal}</Text>}
    </RecordFrame>
  );
};

export { BoardRow };
