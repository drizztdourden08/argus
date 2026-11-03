/* @layer renderer-app @kind component */
import { Tag } from '@drizztdourden08/tessera';
import { MASTER } from '../builds.constants';
import type { LauncherPartProps } from '../builds-props.type';

const LinkChip = (props: LauncherPartProps) => {
  const { launcher } = props.view;
  if (launcher.retiredAt) {
    const item = launcher.retiredByItem ?? launcher.for;
    if (!item) return null;
    return (
      <Tag pill icon="link" href={`#${item}`} title={`Open ${item} on the Kanban tab`}>
        {launcher.retiredByItem ? `closed with ${item}` : `was for ${item}`}
      </Tag>
    );
  }
  if (launcher.relink) {
    return <Tag variant="urgency" color="warning" title="This build was linked by the migration to a guess: link it to the item it serves (coord launcher link <name> I-n)">needs a new link</Tag>;
  }
  if (!launcher.for && launcher.name !== MASTER && !launcher.master) {
    return <Tag variant="urgency" color="warning" title="Every build except master serves an item: link it (coord launcher link <name> I-n)">no item</Tag>;
  }
  return null;
};

export { LinkChip };
