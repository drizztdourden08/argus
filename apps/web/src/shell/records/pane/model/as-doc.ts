/* @layer renderer-app @kind logic */
import type { PaneDoc } from '../pane.type';

const asDoc = (doc: object): PaneDoc => Object.fromEntries(Object.entries(doc));

export { asDoc };
