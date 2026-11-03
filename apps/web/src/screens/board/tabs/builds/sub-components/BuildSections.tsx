/* @layer renderer-app @kind component */
import { MASTER_NOTE, MASTER_TITLE, NEEDS_LINK_NOTE, NEEDS_LINK_TITLE } from '../builds.constants';
import type { BuildSectionsProps } from '../builds-props.type';
import { BuildGroup } from './BuildGroup';
import { GroupTitle } from './GroupTitle';
import { ItemHead } from './ItemHead';

const BuildSections = (props: BuildSectionsProps) => {
  const { groups, boardKey, runs, onLog } = props;
  const shared = { boardKey, runs, onLog };
  return (
    <>
      {groups.master.length > 0 && (
        <BuildGroup id="master" head={<GroupTitle title={MASTER_TITLE} note={MASTER_NOTE} />} views={groups.master} label={MASTER_TITLE} {...shared} />
      )}
      {groups.items.map((group) => (
        <BuildGroup key={group.item.id} id={group.item.id} head={<ItemHead item={group.item} />} views={group.views} label={`Builds for ${group.item.id}`} {...shared} />
      ))}
      {groups.needsLink.length > 0 && (
        <BuildGroup id="needs-link" head={<GroupTitle title={NEEDS_LINK_TITLE} note={NEEDS_LINK_NOTE} />} views={groups.needsLink} label={NEEDS_LINK_TITLE} {...shared} />
      )}
    </>
  );
};

export { BuildSections };
