/* @layer renderer-app @kind component */
import { DateText, Inline, ListItemRow, Tag, Text } from '@drizztdourden08/tessera';
import { ProjectLink } from '../../projects';
import { isFresh } from '../../seen';
import type { ActivityRowProps } from '../activity.type';
import { kindLook, LEVEL_LOOK, levelOf } from '../model';
import { kindOf } from '../model/kind-of';

const ActivityRow = (props: ActivityRowProps) => {
  const { item, freshSince } = props;
  const level = levelOf(item);
  const kind = kindOf(item);
  const project = item.project ?? '';
  return (
    <ListItemRow
      icon={<Tag {...kindLook(kind)}>{kind}</Tag>}
      name={item.text ?? ''}
      nameAction={isFresh(item.at, freshSince) ? <Tag variant="urgency" color="info">new</Tag> : undefined}
      meta={(
        <Inline gap="xs" align="center">
          {project ? <ProjectLink project={project} /> : null}
          {item.thread ? <Text variant="caption" tone="muted">{project ? `/ ${item.thread}` : item.thread}</Text> : null}
        </Inline>
      )}
      columns={[
        { primary: <Tag {...LEVEL_LOOK[level]}>{level}</Tag> },
        { primary: item.at ? <DateText value={item.at} format="relative" tooltip /> : '', align: 'end' },
      ]}
    />
  );
};

export { ActivityRow };
