/* @layer renderer-app @kind component */
import { Box, Icon, IconButton } from '@drizztdourden08/tessera';
import type { TreeLeadProps } from '../review-tree-props.type';

const kindIcon = (props: TreeLeadProps) => {
  if (props.ghost) return 'plus';
  if (props.row.kind === 'chat') return 'message-square';
  return props.collapsed ? 'folder' : 'folder-open';
};

const TreeLead = (props: TreeLeadProps) => {
  const { kids, collapsed, onToggle } = props;
  const label = collapsed ? 'Expand' : 'Collapse';
  return (
    <>
      {kids ? (
        <IconButton variant="bare" size="2xs" className="review-tree-slot" label={label} tooltip={label} aria-expanded={!collapsed} onClick={onToggle}>
          <Icon name={collapsed ? 'chevron-right' : 'chevron-down'} />
        </IconButton>
      ) : (
        <Box as="span" className="review-tree-slot" aria-hidden="true" />
      )}
      <Box as="span" className="review-tree-icon"><Icon name={kindIcon(props)} /></Box>
    </>
  );
};

export { TreeLead };
