/* @layer renderer-app @kind test */
import { Button, ToastStack } from '@drizztdourden08/tessera';
import { act, render } from '@testing-library/react';
import { useRef } from 'react';
import { MemoryRouter, useLocation } from 'react-router';
import { DataProvider, PORTFOLIO_SCOPE } from '../../src/data/react';
import { createScopeStreams } from '../../src/data/stream';
import { useSendDown } from '../../src/screens/overview/boards/behavior/useSendDown';
import { SendDownPopover } from '../../src/screens/overview/boards/sub-components/SendDownPopover';
import { BoardCard } from '../../src/screens/overview/boards/sub-components/BoardCard';
import type { BoardLive } from '../../src/screens/overview/boards/model';
import type { BoardItem } from '../../src/data/boards';
import { snapshot } from '../helpers/snapshot';
import { createFakeSources } from '../helpers/fake-source';
import { NOW, record } from './helpers';

const QUIET: BoardLive = { fault: null, stalls: 0, syncDue: false };

interface HostProps {
  item: BoardItem;
  live: BoardLive;
}

const Where = () => <output data-testid="where">{useLocation().search}</output>;

const Host = (props: HostProps) => {
  const { item, live } = props;
  const send = useSendDown();
  const anchor = useRef<HTMLButtonElement>(null);
  const shown = record(item.doc.key, item.card, item.doc);
  return (
    <>
      <BoardCard record={shown} card={item.card} live={live} now={NOW} send={send} sync={null} onLog={() => undefined} />
      <Where />
      <Button ref={anchor} onClick={() => send.open('all')}>Send to all</Button>
      <SendDownPopover send={send} owner="all" anchorRef={anchor} />
    </>
  );
};

const renderCard = (item: BoardItem, live: BoardLive = QUIET, counts: Record<string, object> = {}) => {
  const fake = createFakeSources();
  const view = render(
    <DataProvider streams={createScopeStreams(fake.open)}>
      <MemoryRouter>
        <Host item={item} live={live} />
      </MemoryRouter>
      <ToastStack />
    </DataProvider>,
  );
  act(() => {
    fake.sources.find((source) => source.scope === PORTFOLIO_SCOPE)?.emit('snapshot', snapshot(PORTFOLIO_SCOPE, 'meta', { worktrees: { counts } }));
  });
  return view;
};

export { QUIET, renderCard };
