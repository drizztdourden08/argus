/* @layer renderer-app @kind component */
import { useRecordPane } from '../../../shell';
import { RecordDrawer, WIDE_KEY, nonEmpty, useNow, useStoredFlag } from '../shared';
import { useThreadOps } from './behavior/useThreadOps';
import { useThreadOrder } from './behavior/useThreadOrder';
import { useThreadView } from './behavior/useThreadView';
import type { ThreadDrawerProps } from './ThreadDrawer.type';
import { ThreadBody } from './sub-components/ThreadBody';
import { ThreadEyebrow } from './sub-components/ThreadEyebrow';
import { ThreadTools } from './sub-components/ThreadTools';
import './ThreadDrawer.css';

const ThreadDrawer = (props: ThreadDrawerProps) => {
  const { boardKey, id, onClose, onPurge } = props;
  const view = useThreadView(boardKey, id);
  const ops = useThreadOps(boardKey);
  const order = useThreadOrder(boardKey);
  const now = useNow(id !== null);
  const pane = useRecordPane();
  const [wide, setWide] = useStoredFlag(WIDE_KEY);
  const { thread } = view;
  const openItem = (itemId: string) => {
    onClose();
    pane.open(itemId);
  };
  return (
    <RecordDrawer
      open={thread !== null}
      label="Thread detail"
      wide={wide}
      eyebrow={thread && <ThreadEyebrow thread={thread} counts={view.counts} />}
      title={thread ? nonEmpty(thread.session, thread.id) : ''}
      tools={thread && <ThreadTools boardKey={boardKey} thread={thread} paused={view.paused} order={order} ops={ops} wide={wide} onWide={() => setWide(!wide)} onPurge={onPurge} />}
      onClose={onClose}
    >
      {thread && <ThreadBody view={{ ...view, thread }} now={now} ops={ops} onOpenItem={openItem} />}
    </RecordDrawer>
  );
};

export { ThreadDrawer };
