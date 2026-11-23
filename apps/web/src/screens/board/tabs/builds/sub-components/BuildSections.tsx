/* @layer renderer-app @kind component */
import { ItemGroupSections } from '../../../item-groups';
import type { ItemGroups } from '../../../item-groups';
import { NEEDS_LINK_TITLE } from '../builds.constants';
import type { BuildSectionsProps } from '../builds-props.type';
import type { LauncherView } from '../model';
import { BuildFlatSections } from './BuildFlatSections';
import { LauncherList } from './LauncherList';
import { MasterGroup } from './MasterGroup';

const BuildSections = (props: BuildSectionsProps) => {
  const { groups, boardKey, runs, threads, group, views, compact } = props;
  if (group !== 'item') return <BuildFlatSections groups={groups} group={group} views={views} boardKey={boardKey} runs={runs} compact={compact} />;
  const shown: ItemGroups<LauncherView> = {
    groups: groups.items.map((group) => ({ item: group.item, rows: group.views })),
    none: groups.needsLink,
    closed: [],
  };
  return (
    <ItemGroupSections
      groups={shown}
      boardKey={boardKey}
      threads={threads}
      label="Builds"
      noneTitle={NEEDS_LINK_TITLE}
      noneFlag
      lead={groups.master.length > 0 ? <MasterGroup views={groups.master} boardKey={boardKey} runs={runs} compact={compact} /> : undefined}
    >
      {(rows) => <LauncherList views={rows} label="Builds" boardKey={boardKey} runs={runs} compact={compact} />}
    </ItemGroupSections>
  );
};

export { BuildSections };
