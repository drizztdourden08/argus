/* @layer renderer-app @kind logic */
import type { HandoverDoc } from '../../../../../data/types';

const handoverTitle = (doc: HandoverDoc): string => (doc.thread ? `${doc.board ?? ''} / ${doc.thread}` : (doc.board ?? doc.id ?? ''));

export { handoverTitle };
