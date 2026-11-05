/* @layer renderer-app @kind component */
import { RecordHeader } from '@drizztdourden08/tessera';
import { TypeIcon } from '../../../../../shell';
import type { TreeRowProps } from '../tree.type';
import { TreeBadges } from './TreeBadges';

const TreeLabel = (props: TreeRowProps) => {
  const { node } = props;
  return <RecordHeader density="row" icon={<TypeIcon kind={node.typeId} />} code={node.id} title={node.title} content={<TreeBadges {...props} />} />;
};

export { TreeLabel };
