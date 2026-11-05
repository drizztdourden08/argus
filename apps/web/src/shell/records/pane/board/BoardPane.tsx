/* @layer renderer-app @kind component */
import { RecordDrawer } from '../../../../screens/board/shared';
import { useRecordPane } from '../../behavior/useRecordPane';
import { useBoardRecordData } from './behavior/useBoardRecordData';
import { BOARD_LABEL } from './board.constants';
import type { BoardPaneProps } from './board.type';
import { BoardFrame } from './BoardFrame';

const BoardPane = (props: BoardPaneProps) => {
  const { target } = props;
  const pane = useRecordPane();
  const { item, ready } = useBoardRecordData(target);
  if (item) return <BoardFrame item={item} />;
  if (!ready) return null;
  return (
    <RecordDrawer open label={BOARD_LABEL} wide={false} eyebrow={target.code} title="Not found" onClose={pane.close}>
      {`${target.code} is not on the portfolio.`}
    </RecordDrawer>
  );
};

export { BoardPane };
