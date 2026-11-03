/* @layer renderer-app @kind component */
import { DepTag } from '../../../../../shell';
import { ATTENTION_OF } from '../attention';
import type { DepTagListProps } from '../kanban-props.type';
import { depTags } from '../model';

const DepTagList = (props: DepTagListProps) => {
  const { item, now } = props;
  return (
    <>
      {depTags(item, now).map((tag) => <DepTag key={tag.key} color={tag.color} icon={ATTENTION_OF.dependency.icon} title={tag.title}>{tag.label}</DepTag>)}
    </>
  );
};

export { DepTagList };
