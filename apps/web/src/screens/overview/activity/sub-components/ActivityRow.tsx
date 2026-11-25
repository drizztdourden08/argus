/* @layer renderer-app @kind component */
import { Flex, Tag, Text } from '@drizztdourden08/tessera';
import { Age } from '../../../../shell';
import { ProjectLink } from '../../projects';
import { leadOf, OverviewRow, recordOfText } from '../../records';
import { isFresh } from '../../seen';
import type { ActivityRowProps } from '../activity.type';
import { LEVEL_TONE, levelOf } from '../model';
import { kindOf } from '../model/kind-of';

const ActivityRow = (props: ActivityRowProps) => {
  const { item, freshSince } = props;
  const level = levelOf(item);
  const kind = kindOf(item);
  const project = item.project ?? '';
  const target = recordOfText(item.text ?? '', project, kind);
  return (
    <OverviewRow
      target={target}
      assignee={leadOf(project)}
      title={item.text ?? ''}
      chips={target.code === kind ? [] : [{ id: 'kind', text: kind, title: `${kind} · ${level}` }]}
      trailing={
        <Flex gap="xs" align="center">
          <Text variant="caption" tone={LEVEL_TONE[level]}>{level}</Text>
          {project ? <ProjectLink project={project} chip /> : null}
          {item.thread ? <Text variant="caption">{`/ ${item.thread}`}</Text> : null}
          {isFresh(item.at, freshSince) ? <Tag variant="urgency" color="info" pill>new</Tag> : null}
          {item.at ? <Text variant="caption" tone="muted"><Age value={item.at} ago /></Text> : null}
        </Flex>
      }
    />
  );
};

export { ActivityRow };
