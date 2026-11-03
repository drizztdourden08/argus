/* @layer renderer-app @kind component */
import { CheckLogOf } from '../checks/CheckLogOf';
import type { RecordBodyProps } from '../pane.type';
import { RunLog } from '../runs/RunLog';

const LogTab = (props: RecordBodyProps) => (props.data.target.collection === 'checks' ? <CheckLogOf {...props} /> : <RunLog {...props} />);

export { LogTab };
