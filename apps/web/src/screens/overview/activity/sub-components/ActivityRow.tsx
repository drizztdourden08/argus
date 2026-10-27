/* @layer renderer-app @kind component */
import { Tag, Text } from '@drizztdourden08/tessera';
import { ProjectLink } from '../../projects';
import { isFresh } from '../../seen';
import { FeedRow } from '../../sub-components/FeedRow';
import type { ActivityRowProps } from '../activity.type';
import { kindIcon, LEVEL_TONE, levelOf } from '../model';
import { kindOf } from '../model/kind-of';

const ActivityRow = (props: ActivityRowProps) => {
  const { item, freshSince } = props;
  const level = levelOf(item);
  const kind = kindOf(item);
  const project = item.project ?? '';
  return (
    <FeedRow icon={kindIcon(kind)} tone={LEVEL_TONE[level]} title={`${kind} · ${level}`} at={item.at}>
      <Text>
        <Text variant="overline" tone={LEVEL_TONE[level]}>{kind}</Text>
        {` ${item.text ?? ''} `}
        {project ? <ProjectLink project={project} chip /> : null}
        {item.thread ? <Text variant="caption">{` ${project ? '/ ' : ''}${item.thread}`}</Text> : null}
        {isFresh(item.at, freshSince) ? <> <Tag variant="urgency" color="info">new</Tag></> : null}
      </Text>
    </FeedRow>
  );
};

export { ActivityRow };
