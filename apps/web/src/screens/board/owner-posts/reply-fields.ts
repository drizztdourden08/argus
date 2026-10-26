/* @layer renderer-app @kind logic */
import type { OwnerFields } from '../../../data/posts';
import type { PostTarget, ReplyKind } from './owner-posts.type';

const replyFields = (entry: PostTarget, kind: ReplyKind, text: string): OwnerFields => ({
  kind,
  text,
  replyTo: entry.id,
  item: entry.item ?? null,
  thread: entry.thread ?? null,
});

export { replyFields };
