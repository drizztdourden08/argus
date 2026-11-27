/* @layer renderer-app @kind component */
import type { ProcessDocProps } from '../switch.type';
import { ProcessWizard } from './ProcessWizard';

const ProcessDetail = (props: ProcessDocProps) => <ProcessWizard doc={props.doc} />;

export { ProcessDetail };
