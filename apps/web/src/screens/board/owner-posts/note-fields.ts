/* @layer renderer-app @kind logic */
import type { OwnerFields } from '../../../data/posts';
import type { WorkItem } from '../../../data/types';
import type { ReplyKind } from './owner-posts.type';

const noteFields = (item: WorkItem, kind: ReplyKind, text: string): OwnerFields => ({
  kind,
  text,
  item: item.id,
  thread: item.thread,
});

export { noteFields };
