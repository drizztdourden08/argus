/* @layer renderer-app @kind component */
import type { NoticeRowProps } from '../OwnerFeed.type';
import { DismissButton } from './DismissButton';
import { EntryRow } from './EntryRow';

const NoticeRow = (props: NoticeRowProps) => {
  const { entry, ctx } = props;
  return <EntryRow type="notice" entry={entry} ctx={ctx} trailing={<DismissButton entry={entry} actions={ctx.actions} />} />;
};

export { NoticeRow };
