/* @layer renderer-app @kind component */
import type { PaneBodyProps } from '../pane.type';
import { RunPanel } from '../run/RunPanel';

const RunTab = (props: PaneBodyProps) => <RunPanel data={props.data} />;

export { RunTab };
