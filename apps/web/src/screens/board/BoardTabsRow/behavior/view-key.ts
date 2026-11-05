/* @layer renderer-app @kind logic */
const viewKey = (tab: string, part: 'filters' | 'group' | 'sort'): string => `argus.tab.${tab}.${part}`;

export { viewKey };
