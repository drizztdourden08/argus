/* @layer renderer-app @kind logic */
import { PANE_TABS } from '../pane.constants';

const kindTabs = (face: { tabs: readonly string[] }): string[] => face.tabs.filter((id) => Object.hasOwn(PANE_TABS, id));

export { kindTabs };
