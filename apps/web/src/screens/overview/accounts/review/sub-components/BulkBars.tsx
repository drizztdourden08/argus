/* @layer renderer-app @kind component */
import { bulkIds } from '../model';
import type { ChatOp } from '../review.type';
import type { BulkBarsProps } from '../review-props.type';
import { BulkBar } from './BulkBar';

const BulkBars = (props: BulkBarsProps) => {
  const { accounts, rows, selection, onStage, onMove, onClear } = props;
  return (
    <>
      {accounts.map((account) => {
        const ids = bulkIds(rows, account.key, selection[account.key] ?? [], 'any');
        if (ids.length === 0) return null;
        const active = bulkIds(rows, account.key, ids, 'active');
        const archived = bulkIds(rows, account.key, ids, 'archived');
        const stage = (op: ChatOp, list: readonly string[]) => onStage(list.map((sessionId) => ({ op, account: account.key, sessionId })));
        return (
          <BulkBar
            key={account.key}
            label={account.label ?? account.key}
            total={ids.length}
            archive={active.length}
            unarchive={archived.length}
            remove={archived.length}
            onArchive={() => stage('archive', active)}
            onUnarchive={() => stage('unarchive', archived)}
            onMove={() => onMove(account.key, ids)}
            onDelete={() => stage('delete_chat', archived)}
            onClear={() => onClear(account.key)}
          />
        );
      })}
    </>
  );
};

export { BulkBars };
