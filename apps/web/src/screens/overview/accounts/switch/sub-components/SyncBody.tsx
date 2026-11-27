/* @layer renderer-app @kind component */
import type { StepBodyProps } from '../switch.type';
import { SyncBlock } from './SyncBlock';

const SyncBody = (props: StepBodyProps) => <SyncBlock sync={props.doc.sync} />;

export { SyncBody };
