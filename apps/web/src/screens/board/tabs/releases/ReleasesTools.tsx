/* @layer renderer-app @kind component */
import { TabFilterBar, TabGroupSort, useTabTools } from '../../BoardTabsRow';
import type { TabField } from '../../BoardTabsRow';
import { RELEASES_ORDERS, RELEASES_TAB } from './releases.constants';
import type { ReleasesFilters, ReleasesToolsProps } from './releases.type';

const ReleasesTools = (props: ReleasesToolsProps) => {
  const { boardKey, view, channels } = props;
  const sort = { options: RELEASES_ORDERS, order: view.order, direction: view.direction, onChange: view.setSort };
  const fields: TabField<ReleasesFilters>[] = [
    {
      kind: 'single',
      path: 'channel',
      label: 'Channel',
      options: channels.map((channel) => ({ value: channel.channel, label: channel.channel })),
      read: (filters) => filters.channel,
      write: (channel) => ({ channel }),
    },
  ];
  useTabTools(RELEASES_TAB, [{ id: 'releases-view', priority: 5, node: <TabGroupSort sort={sort} /> }]);
  return (
    <TabFilterBar tab={RELEASES_TAB} boardKey={boardKey} fields={fields} filters={view.filters} patchFilters={view.patchFilters} placeholder="Search channel or version" />
  );
};

export { ReleasesTools };
