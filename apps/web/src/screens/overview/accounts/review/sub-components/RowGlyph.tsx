/* @layer renderer-app @kind component */
import { Icon, IconButton } from '@drizztdourden08/tessera';
import type { RowGlyphProps } from '../review-props.type';

const RowGlyph = (props: RowGlyphProps) => {
  const { row, collapsed, onToggle } = props;
  if (row.kind === 'folder') {
    return (
      <>
        <IconButton variant="bare" size="2xs" label={collapsed ? 'Expand folder' : 'Collapse folder'} aria-expanded={!collapsed} onClick={onToggle}>
          <Icon name={collapsed ? 'chevron-right' : 'chevron-down'} />
        </IconButton>
        <Icon name={collapsed ? 'folder' : 'folder-open'} />
      </>
    );
  }
  return <Icon name={row.depth >= 2 ? 'arrow-right' : 'message-square'} />;
};

export { RowGlyph };
