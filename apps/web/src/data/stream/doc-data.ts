/* @layer renderer-app @kind logic */
import type { DocData } from '../api';
import type { WireDoc } from './stream.type';

const docData = (doc: WireDoc): DocData => doc.data ?? doc;

export { docData };
