/* @layer renderer-app @kind component */
import { RecordDrawer, WIDE_KEY, useStoredFlag } from '../../../screens/board/shared';
import { neighbour } from '../../../screens/board/tabs/kanban/model';
import { useOrderedIds } from '../behavior/useOrderedIds';
import { usePagerKeys } from './behavior/usePagerKeys';
import { DRAWER_FALLBACK, DRAWER_LABELS } from './model/pane-model.constants';
import { ItemSoftTools } from './ItemSoftTools';
import type { RecordFrameProps } from './pane.type';
import { RecordBody } from './RecordBody';
import { RecordEyebrow } from './RecordEyebrow';
import { RecordPaneTools } from './RecordPaneTools';
import { RecordTitle } from './RecordTitle';

const RecordFrame = (props: RecordFrameProps) => {
  const { data, pane } = props;
  const [wide, setWide] = useStoredFlag(WIDE_KEY);
  const ids = useOrderedIds();
  const current = ids.includes(data.code) ? data.code : data.target.id;
  const previous = neighbour(ids, current, -1);
  const next = neighbour(ids, current, 1);
  usePagerKeys(previous, next, pane.step);
  return (
    <RecordDrawer
      open
      label={DRAWER_LABELS[data.target.collection] ?? DRAWER_FALLBACK}
      wide={wide}
      eyebrow={<RecordEyebrow data={data} />}
      title={<RecordTitle data={data} />}
      tools={
        <RecordPaneTools code={data.code} pane={pane} previous={previous} next={next} wide={wide} onWide={() => setWide(!wide)}>
          <ItemSoftTools data={data} />
        </RecordPaneTools>
      }
      onClose={pane.close}
    >
      <RecordBody data={data} />
    </RecordDrawer>
  );
};

export { RecordFrame };
