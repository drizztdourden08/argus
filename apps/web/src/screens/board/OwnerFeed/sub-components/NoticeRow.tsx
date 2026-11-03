/* @layer renderer-app @kind component */
import { RecordRow, useRecordPane } from '../../../../shell';
import { formatAgo } from '../../../../format';
import { FEED_KIND } from '../../feed-model';
import { cardFacts } from '../behavior/card-facts';
import { feedMenu } from '../behavior/feed-menu';
import type { NoticeRowProps } from '../OwnerFeed.type';
import { DismissButton } from './DismissButton';

const NoticeRow = (props: NoticeRowProps) => {
  const { entry, ctx } = props;
  const pane = useRecordPane();
  const card = { type: 'notice', at: entry.createdAt, data: entry } as const;
  const facts = cardFacts(card);
  const open = () => pane.open(facts.code);
  return (
    <RecordRow
      head={{
        kind: FEED_KIND.notice,
        code: facts.code,
        assignee: facts.assignee,
        boardKey: ctx.boardKey,
        threads: ctx.threads,
        menu: feedMenu(card, facts, ctx.actions, open),
        menuLabel: `More actions for ${facts.code}`,
      }}
      title={facts.title}
      onOpen={open}
      age={formatAgo(entry.createdAt, ctx.now)}
      trailing={<DismissButton entry={entry} actions={ctx.actions} />}
    />
  );
};

export { NoticeRow };
