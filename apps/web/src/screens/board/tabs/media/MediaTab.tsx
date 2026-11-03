/* @layer renderer-app @kind component */
import { EmptyState, Flex, MediaGrid, MediaViewer, Text } from '@drizztdourden08/tessera';
import { useMemo, useState } from 'react';
import type { Item } from '../../../../data/kanban';
import { shotsOf } from '../../../../data/media';
import { useCollection } from '../../../../data/react';
import { Panel } from '../../../../shell';
import { EMPTY_HINT, EMPTY_TEXT, LOADING_TEXT } from './media.constants';
import type { MediaTabProps } from './media.type';
import { gridItems, viewerItems } from './model';
import { ItemLinks } from './sub-components/ItemLinks';

const MediaTab = ({ boardKey }: MediaTabProps) => {
  const items = useCollection<Item>(boardKey, 'items');
  const shots = useMemo(() => shotsOf(items.docs), [items.docs]);
  const [at, setAt] = useState<number | null>(null);
  if (!items.ready) return <Text tone="muted">{LOADING_TEXT}</Text>;
  return (
    <Panel title="Media" note={shots.length > 0 ? `${shots.length} ${shots.length === 1 ? 'screenshot' : 'screenshots'}` : undefined}>
      {shots.length > 0 ? (
        <Flex direction="column" gap="md">
          <ItemLinks boardKey={boardKey} shots={shots} />
          <MediaGrid aria-label="Screenshots of every item" items={gridItems(shots)} onOpen={setAt} />
        </Flex>
      ) : (
        <EmptyState message={EMPTY_TEXT} hint={EMPTY_HINT} />
      )}
      <MediaViewer open={at !== null} items={viewerItems(shots)} index={at ?? 0} onIndexChange={setAt} onClose={() => setAt(null)} />
    </Panel>
  );
};

export { MediaTab };
