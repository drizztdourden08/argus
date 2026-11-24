/* @layer renderer-app @kind constants */
import type { TabItem } from '@drizztdourden08/tessera';

const PACKAGE_TABS: TabItem[] = [
  { id: 'builds', label: 'Local builds' },
  { id: 'cache', label: 'Download cache' },
];

const PACKAGE_SENTENCES: Readonly<Record<string, string>> = {
  builds: 'Packages built from your own repos and the projects that use them.',
  cache: 'Package and Electron versions downloaded to this machine, and what can be freed.',
};

const PACKAGE_TAB_KEY = 'argus.tab.packages.sub';

const DEFAULT_PACKAGE_TAB = 'builds';

export { DEFAULT_PACKAGE_TAB, PACKAGE_SENTENCES, PACKAGE_TAB_KEY, PACKAGE_TABS };
