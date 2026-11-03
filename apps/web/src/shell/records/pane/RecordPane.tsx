/* @layer renderer-app @kind component */
import { RecordDrawer } from '../../../screens/board/shared';
import { useRecordPane } from '../behavior/useRecordPane';
import { useRecordData } from './behavior/useRecordData';
import { DRAWER_FALLBACK, DRAWER_LABELS } from './model/pane-model.constants';
import type { RecordPaneProps } from './pane.type';
import { RecordFrame } from './RecordFrame';

const RecordPane = (props: RecordPaneProps) => {
  const { boardKey, target } = props;
  const pane = useRecordPane();
  const { data, ready } = useRecordData(boardKey, target);
  if (data) return <RecordFrame data={data} pane={pane} />;
  if (!ready) return null;
  return (
    <RecordDrawer open label={DRAWER_LABELS[target.collection] ?? DRAWER_FALLBACK} wide={false} eyebrow={target.code} title="Not found" onClose={pane.close}>
      {`${target.code} is not on the board ${boardKey}.`}
    </RecordDrawer>
  );
};

export { RecordPane };
