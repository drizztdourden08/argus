/* @layer renderer-app @kind component */
import { SoftTools } from '../../../screens/board/shared';
import type { RecordDataProps } from './pane.type';

const ItemSoftTools = (props: RecordDataProps) => {
  const { item, ops } = props.data;
  if (!item) return null;
  const archived = Boolean(item.item.archivedAt);
  return <SoftTools archived={archived} onArchive={() => void ops.soft(archived ? 'unarchive' : 'archive', item.item)} onDelete={() => void ops.soft('delete', item.item)} />;
};

export { ItemSoftTools };
