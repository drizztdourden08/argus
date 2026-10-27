/* @layer renderer-app @kind component */
import { Drawer } from '@drizztdourden08/tessera';
import { useNavigate } from 'react-router';
import { WIDE_KEY, nonEmpty, useNow, useStoredFlag } from '../shared';
import { useThreadOps } from './behavior/useThreadOps';
import { useThreadOrder } from './behavior/useThreadOrder';
import { useThreadView } from './behavior/useThreadView';
import type { ThreadDrawerProps } from './ThreadDrawer.type';
import { ThreadBody } from './sub-components/ThreadBody';
import './ThreadDrawer.css';

const ThreadDrawer = (props: ThreadDrawerProps) => {
  const { boardKey, id, onClose, onPurge } = props;
  const view = useThreadView(boardKey, id);
  const ops = useThreadOps(boardKey);
  const order = useThreadOrder(boardKey);
  const now = useNow(id !== null);
  const navigate = useNavigate();
  const [wide, setWide] = useStoredFlag(WIDE_KEY);
  const { thread } = view;
  const openItem = (itemId: string) => {
    onClose();
    void navigate({ hash: itemId }, { replace: true });
  };
  return (
    <Drawer
      open={thread !== null}
      onClose={onClose}
      size={wide ? 'xl' : 'md'}
      label="Thread detail"
      title={thread ? nonEmpty(thread.session, thread.id) : ''}
      subtitle={id ?? undefined}
    >
      {thread && <ThreadBody view={{ ...view, thread }} order={order} now={now} ops={ops} wide={wide} onWide={() => setWide(!wide)} onPurge={onPurge} onOpenItem={openItem} />}
    </Drawer>
  );
};

export { ThreadDrawer };
