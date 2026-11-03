/* @layer renderer-app @kind component */
import { List, ListItem, Small, Tag } from '@drizztdourden08/tessera';
import { Age } from '../../../../shell';
import type { NoticeListProps } from '../BoardHeader.type';

const NoticeList = (props: NoticeListProps) => (
  <List aria-label="Notices" className="board-notices__list">
    {props.notices.map((notice) => (
      <ListItem key={notice.id} bare className="board-notices__row">
        <Small tone="muted">{notice.at ? <Age value={notice.at} /> : null}</Small>
        <Tag>{notice.reason ?? 'notice'}</Tag>
        <Small tone="info" title={notice.thread ?? undefined}>{notice.thread}</Small>
        <Small className="board-notices__text" title={notice.text}>{notice.text}</Small>
      </ListItem>
    ))}
  </List>
);

export { NoticeList };
