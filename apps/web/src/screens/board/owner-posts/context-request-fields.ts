/* @layer renderer-app @kind logic */
import type { OwnerFields } from '../../../data/posts';
import type { PostTarget } from './owner-posts.type';

const contextRequestFields = (question: PostTarget): OwnerFields => ({
  kind: 'followup',
  text: `Please add context: what is this about and why does it need me? (${question.id})`,
  item: question.item ?? null,
  thread: question.thread ?? null,
});

export { contextRequestFields };
