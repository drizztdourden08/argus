/* @layer renderer-app @kind component */
import { Icon, IconButton } from '@drizztdourden08/tessera';
import type { SoftToolsProps } from '../shared.type';

const SoftTools = (props: SoftToolsProps) => {
  const { archived, onArchive, onDelete } = props;
  return (
    <>
      <IconButton label={archived ? 'Unarchive' : 'Archive'} size="xs" variant="ghost" onClick={onArchive}>
        <Icon name={archived ? 'rotate-ccw' : 'archive'} />
      </IconButton>
      <IconButton label="Delete" title="Delete (Undo stays a few seconds)" size="xs" variant="ghost" tone="danger" onClick={onDelete}>
        <Icon name="trash-2" />
      </IconButton>
    </>
  );
};

export { SoftTools };
