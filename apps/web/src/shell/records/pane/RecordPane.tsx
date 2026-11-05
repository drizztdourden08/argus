/* @layer renderer-app @kind component */
import { BoardPane } from './board/BoardPane';
import { ItemRecordPane } from './ItemRecordPane';
import type { RecordPaneProps } from './pane.type';

const RecordPane = (props: RecordPaneProps) => (props.target.collection === 'boards' ? <BoardPane target={props.target} /> : <ItemRecordPane {...props} />);

export { RecordPane };
