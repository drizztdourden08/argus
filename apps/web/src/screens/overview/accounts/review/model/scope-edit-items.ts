/* @layer renderer-app @kind logic */
import type { MenuItem } from '@drizztdourden08/tessera';
import type { ReviewEdit } from '../../../../../data/types';
import type { ScopeMenuArgs } from '../review-tree-props.type';
import type { ChatOp } from '../review.type';
import { moveItems } from './move-items';
import { scopeChats } from './scope-chats';
import { scopeOf } from './scope-of';

const scopeEditItems = (args: ScopeMenuArgs): MenuItem[] => {
  const { row, account, env } = args;
  const scope = scopeOf(row, env.scopes);
  const active = scopeChats(scope, account, 'active');
  const archived = scopeChats(scope, account, 'archived');
  const all = scopeChats(scope, account, 'any');
  const stage = (op: ChatOp, ids: readonly string[]) => (): void => env.onStage(ids.map((sessionId): ReviewEdit => ({ op, account, sessionId })));
  const moveAll = (folder: string | null): void => env.onStage(all.map((sessionId): ReviewEdit => ({ op: 'move_chat', account, sessionId, folder })));
  const items: (MenuItem & { count: number })[] = [
    { id: 'archive-all', label: `Archive all (${active.length})`, icon: 'archive', count: active.length, onSelect: stage('archive', active) },
    { id: 'unarchive-all', label: `Unarchive all (${archived.length})`, icon: 'rotate-ccw', count: archived.length, onSelect: stage('unarchive', archived) },
    { id: 'move-all', label: `Move all to... (${all.length})`, icon: 'folder', count: all.length, children: moveItems(env.folders.get(account) ?? [], moveAll) },
    { id: 'delete-archived', label: `Delete archived ones (${archived.length})`, icon: 'trash-2', tone: 'danger', count: archived.length, onSelect: stage('delete_chat', archived) },
  ];
  return items.filter((item) => item.count > 0).map(({ count: _count, ...item }): MenuItem => item);
};

export { scopeEditItems };
