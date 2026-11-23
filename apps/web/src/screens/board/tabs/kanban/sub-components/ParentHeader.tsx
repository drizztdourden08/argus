/* @layer renderer-app @kind component */
import { RecordHeader } from '@drizztdourden08/tessera';
import { RecordLink, TypeIcon } from '../../../../../shell';
import type { ParentHeaderProps } from '../kanban-props.type';
import { DEFAULT_TYPE } from '../model';

const ParentHeader = (props: ParentHeaderProps) => {
  const { parent } = props;
  return (
    <RecordHeader
      density="row"
      icon={<TypeIcon kind={parent.type ?? DEFAULT_TYPE} />}
      code={<RecordLink id={parent.id} />}
      title={parent.title}
    />
  );
};

export { ParentHeader };
