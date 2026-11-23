/* @layer renderer-app @kind component */
import { Pressable, RecordHeader, Tag, Text } from '@drizztdourden08/tessera';
import { itemCode, RecordLink, RecordMenu, TypeIcon } from '../../../../../shell';
import type { EpicHeaderProps } from '../kanban-props.type';
import { DEFAULT_TYPE } from '../model';

const EpicHeader = (props: EpicHeaderProps) => {
  const { parent, count, menu, fold, onOpen } = props;
  const code = itemCode(parent);
  return (
    <RecordHeader
      density="row"
      icon={<TypeIcon kind={parent.type ?? DEFAULT_TYPE} />}
      code={<RecordLink id={code} />}
      title={(
        <Pressable onClick={() => onOpen(code)} aria-label={`Open ${code}: ${parent.title}`}>
          <Text weight={600} clamp={1} smartTimes>{parent.title}</Text>
        </Pressable>
      )}
      content={<Tag title={`${count} in this column`}>{count}</Tag>}
      menu={<RecordMenu groups={menu} label={`More actions for ${code}`} />}
      chevron={{ expanded: fold.open, onToggle: fold.toggle, controls: fold.controls, label: `${fold.open ? 'Collapse' : 'Expand'} ${parent.title}` }}
    />
  );
};

export { EpicHeader };
