/* @layer renderer-app @kind constants */
import type { FeedKind, NeedCount } from './feed.type';

const NO_NEEDS: NeedCount = { questions: 0, decisions: 0, steps: 0, tests: 0 };

const OP_KINDS = ['create', 'edit', 'move', 'ac', 'archive', 'unarchive', 'delete', 'restore', 'bulk', 'purge', 'build', 'avatar'];

const FEED_META: Record<FeedKind, { label: string; you: boolean }> = {
  decision: { label: 'decision needed', you: true },
  question: { label: 'question', you: true },
  action: { label: 'manual step', you: true },
  test: { label: 'test to run', you: true },
  bundle: { label: 'bundle to test', you: true },
  message: { label: 'message', you: true },
  notice: { label: 'notice', you: false },
};

const FINISHED_ITEM_STATUSES = ['done', 'dropped'];

const CONTEXT_SHOWN_PARAGRAPHS = 2;

export { CONTEXT_SHOWN_PARAGRAPHS, FEED_META, FINISHED_ITEM_STATUSES, NO_NEEDS, OP_KINDS };
