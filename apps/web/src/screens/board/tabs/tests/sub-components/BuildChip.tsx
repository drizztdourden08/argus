/* @layer renderer-app @kind component */
import { Tag } from '@drizztdourden08/tessera';
import { itemCode, useRecordPane } from '../../../../../shell';
import { BUILD_KIND, BUILD_TIP } from '../tests.constants';
import type { BuildChipProps } from '../tests-head-props.type';

const BuildChip = (props: BuildChipProps) => {
  const { build, gone } = props;
  const pane = useRecordPane();
  if (gone) return <Tag variant="urgency" color="warning" icon="triangle-alert" title={gone}>{build}</Tag>;
  return (
    <Tag icon="package" title={BUILD_TIP} aria-label={`Open build ${build}`} onClick={() => pane.open(itemCode({ id: build, type: BUILD_KIND }), 'runs')}>{build}</Tag>
  );
};

export { BuildChip };
