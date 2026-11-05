/* @layer renderer-app @kind component */
import { Icon, IconButton } from '@drizztdourden08/tessera';
import { useBoardWrite } from './behavior/useBoardWrite';
import type { BoardArchiveToolProps } from './board.type';

const BoardArchiveTool = (props: BoardArchiveToolProps) => {
  const { record } = props;
  const write = useBoardWrite();
  const kind = record.archived ? 'unarchive' : 'archive';
  return (
    <IconButton label={record.archived ? 'Unarchive' : 'Archive'} size="xs" variant="ghost" onClick={() => void write({ kind, id: record.id }, `${record.name} ${kind}d.`)}>
      <Icon name={record.archived ? 'rotate-ccw' : 'archive'} />
    </IconButton>
  );
};

export { BoardArchiveTool };
