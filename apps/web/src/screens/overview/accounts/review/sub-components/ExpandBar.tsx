/* @layer renderer-app @kind component */
import { Icon, IconButton, Inline } from '@drizztdourden08/tessera';
import type { ExpandBarProps } from '../review-tree-props.type';

const ExpandBar = (props: ExpandBarProps) => (
  <Inline gap="xs" align="center">
    <IconButton variant="ghost" size="sm" label="Expand all" tooltip="Expand all" onClick={props.onExpandAll}><Icon name="maximize-2" /></IconButton>
    <IconButton variant="ghost" size="sm" label="Collapse all" tooltip="Collapse all" onClick={props.onCollapseAll}><Icon name="minimize-2" /></IconButton>
  </Inline>
);

export { ExpandBar };
