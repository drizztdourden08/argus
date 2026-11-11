/* @layer renderer-app @kind component */
import { folderNames } from '../model';
import type { ReviewDialogsProps } from '../review-props.type';
import { ApplyDialog } from './ApplyDialog';
import { CleanupDialog } from './CleanupDialog';
import { MoveDialog } from './MoveDialog';
import { NameDialog } from './NameDialog';

const ReviewDialogs = (props: ReviewDialogsProps) => {
  const { dialog, accounts, rows, choiceLines, editLines, count, applying, onClose, onStage, onApply } = props;
  if (!dialog) return null;
  const stage = (edits: Parameters<typeof onStage>[0]) => {
    onStage(edits);
    onClose();
  };
  switch (dialog.kind) {
    case 'rename': {
      const { account, folder } = dialog;
      return <NameDialog title="Rename folder" label="Folder name" initial={folder} confirmLabel="Rename" onCancel={onClose} onConfirm={(name) => stage([{ op: 'rename_folder', account, folder, name }])} />;
    }
    case 'new': {
      const { account } = dialog;
      return <NameDialog title="New folder" label="Folder name" initial="" confirmLabel="Create" onCancel={onClose} onConfirm={(name) => stage([{ op: 'create_folder', account, name }])} />;
    }
    case 'move': {
      const { account, ids } = dialog;
      const label = accounts.find((item) => item.key === account)?.label ?? account;
      return (
        <MoveDialog
          account={label}
          count={ids.length}
          folders={folderNames(rows, account)}
          onCancel={onClose}
          onConfirm={(folder) => stage(ids.map((sessionId) => ({ op: 'move_chat', account, sessionId, folder })))}
        />
      );
    }
    case 'cleanup': return <CleanupDialog accounts={accounts} onCancel={onClose} onStage={stage} />;
    case 'apply': return <ApplyDialog choiceLines={choiceLines} editLines={editLines} count={count} busy={applying} onCancel={onClose} onConfirm={onApply} />;
  }
};

export { ReviewDialogs };
