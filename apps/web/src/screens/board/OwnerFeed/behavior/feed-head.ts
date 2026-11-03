/* @layer renderer-app @kind logic */
import type { RecordHeadProps } from '../../../../shell';
import type { FeedCard } from '../../feed-model';
import type { CardFacts, FeedContext } from '../OwnerFeed.type';
import { feedMenu } from './feed-menu';

const feedHead = (card: FeedCard, facts: CardFacts, ctx: FeedContext, open: () => void): RecordHeadProps => ({
  kind: facts.kind,
  code: facts.code,
  assignee: facts.assignee,
  boardKey: ctx.boardKey,
  threads: ctx.threads,
  menu: feedMenu(card, facts, ctx.actions, open),
  menuLabel: `More actions for ${facts.code}`,
});

export { feedHead };
