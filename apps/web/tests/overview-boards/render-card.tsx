/* @layer renderer-app @kind test */
import { Button, ToastStack } from '@drizztdourden08/tessera';
import { render } from '@testing-library/react';
import { useRef } from 'react';
import { MemoryRouter } from 'react-router';
import { DataProvider } from '../../src/data/react';
import { createScopeStreams } from '../../src/data/stream';
import { useSendDown } from '../../src/screens/overview/boards/behavior/useSendDown';
import { SendDownPopover } from '../../src/screens/overview/boards/sub-components/SendDownPopover';
import { BoardCard } from '../../src/screens/overview/boards/sub-components/BoardCard';
import type { BoardLive } from '../../src/screens/overview/boards/model';
import type { BoardItem } from '../../src/data/boards';
import { createFakeSources } from '../helpers/fake-source';
import { NOW, record } from './helpers';

const QUIET: BoardLive = { fault: null, stalls: 0, syncDue: false };

interface HostProps {
  item: BoardItem;
  live: BoardLive;
}

const Host = (props: HostProps) => {
  const { item, live } = props;
  const send = useSendDown();
  const anchor = useRef<HTMLButtonElement>(null);
  const shown = record(item.doc.key, item.card, item.doc);
  return (
    <>
      <BoardCard record={shown} card={item.card} live={live} now={NOW} send={send} sync={null} onLog={() => undefined} />
      <Button ref={anchor} onClick={() => send.open('all')}>Send to all</Button>
      <SendDownPopover send={send} owner="all" anchorRef={anchor} />
    </>
  );
};

const renderCard = (item: BoardItem, live: BoardLive = QUIET) => {
  const fake = createFakeSources();
  return render(
    <DataProvider streams={createScopeStreams(fake.open)}>
      <MemoryRouter>
        <Host item={item} live={live} />
      </MemoryRouter>
      <ToastStack />
    </DataProvider>,
  );
};

export { QUIET, renderCard };
