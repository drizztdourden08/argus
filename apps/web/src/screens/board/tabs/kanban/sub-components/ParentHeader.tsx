/* @layer renderer-app @kind component */
import { RecordHeader } from '@drizztdourden08/tessera';
import { RecordLink, TypeIcon } from '../../../../../shell';
import type { ParentHeaderProps } from '../kanban-props.type';
import { DEFAULT_TYPE } from '../model';

const ParentHeader = (props: ParentHeaderProps) => {
  const { parent, withCode, fold } = props;
  return (
    <RecordHeader
      density="row"
      icon={<TypeIcon kind={parent.type ?? DEFAULT_TYPE} />}
      code={withCode ? <RecordLink id={parent.id} /> : undefined}
      title={parent.title}
      chevron={fold && { expanded: fold.open, onToggle: fold.toggle, controls: fold.controls, label: `${fold.open ? 'Collapse' : 'Expand'} ${parent.title}` }}
    />
  );
};

export { ParentHeader };
