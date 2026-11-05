/* @layer renderer-app @kind logic */
import { useStoredFlag } from '../../shared';
import { FILTERS_OPEN_KEY } from '../BoardTabsRow.constants';

const useFiltersOpen = (tab: string): [boolean, (open: boolean) => void] => useStoredFlag(`${FILTERS_OPEN_KEY}.${tab}`);

export { useFiltersOpen };
