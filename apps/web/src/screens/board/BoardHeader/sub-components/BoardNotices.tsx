/* @layer renderer-app @kind component */
import { Anchored, Badge, Button, Card, Flex, Text } from '@drizztdourden08/tessera';
import { NOTICES_HINT, NOTICES_LABEL } from '../BoardHeader.constants';
import type { BoardNoticesProps } from '../BoardHeader.type';
import { useNotices } from '../behavior/useNotices';
import { NoticeList } from './NoticeList';
import './BoardNotices.css';

const BoardNotices = (props: BoardNoticesProps) => {
  const { notices, anchor, open, toggle, close } = useNotices(props.boardKey);
  const count = notices.length;
  if (count === 0) return null;
  return (
    <>
      <Button ref={anchor} size="sm" variant="ghost" title={NOTICES_HINT} aria-expanded={open} onClick={toggle}>
        {NOTICES_LABEL} <Badge variant="number" color="info" value={count} label={`${count} ${NOTICES_LABEL}`} />
      </Button>
      {open ? (
        <Anchored anchorRef={anchor} placement="bottom-start" onDismiss={close} className="board-notices" role="dialog" aria-label="Board notices">
          <Card>
            <Flex direction="column" gap="xs">
              <Text variant="caption" tone="muted">{NOTICES_HINT}</Text>
              <NoticeList notices={notices} />
            </Flex>
          </Card>
        </Anchored>
      ) : null}
    </>
  );
};

export { BoardNotices };
