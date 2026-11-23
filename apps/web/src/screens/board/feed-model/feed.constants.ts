/* @layer renderer-app @kind constants */
import type { FeedKind, NeedCount } from './feed.type';

const NO_NEEDS: NeedCount = { questions: 0, decisions: 0, steps: 0, tests: 0 };

const OP_KINDS = ['create', 'edit', 'move', 'ac', 'archive', 'unarchive', 'delete', 'restore', 'bulk', 'purge', 'build', 'avatar', 'worktree-link', 'worktree-keep', 'launcher-link', 'launcher-retire'];

const FEED_KIND: Record<FeedKind, string> = {
  decision: 'decision',
  question: 'question',
  action: 'todo',
  test: 'test-case',
  run: 'test-run',
  message: 'message',
  notice: 'notice',
  sent: 'followup',
};

const OWNER_ASSIGNEE = 'owner';

const OWNER_AUTHOR = 'human';

const SENT_KINDS: readonly string[] = ['followup', 'message'];

const FINISHED_ITEM_STATUSES = ['done', 'dropped'];

const CONTEXT_SHOWN_PARAGRAPHS = 2;

export { CONTEXT_SHOWN_PARAGRAPHS,FEED_KIND, FINISHED_ITEM_STATUSES, NO_NEEDS, OP_KINDS, OWNER_ASSIGNEE, OWNER_AUTHOR, SENT_KINDS };
