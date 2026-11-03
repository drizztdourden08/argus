/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import type { NoticeListProps } from '../OwnerFeed.type';
import { NoticeRow } from './NoticeRow';

const NoticeList = (props: NoticeListProps) => {
  const { notices, ctx } = props;
  return (
    <Flex direction="column" gap="xs" role="group" aria-label="Notices">
      <Text variant="overline" tone="muted">Notices · {notices.length}</Text>
      {notices.map((card) => (card.type === 'notice' ? <NoticeRow key={card.data.id} entry={card.data} ctx={ctx} /> : null))}
    </Flex>
  );
};

export { NoticeList };
