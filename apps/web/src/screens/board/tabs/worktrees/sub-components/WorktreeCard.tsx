/* @layer renderer-app @kind component */
import { Flex, Tag, Text } from '@drizztdourden08/tessera';
import { RecordCard } from '../../../../../shell';
import { jobText } from '../model';
import type { WorktreeCardProps } from '../worktrees-props.type';

const WorktreeCard = (props: WorktreeCardProps) => {
  const { head, title, chips, time, job, label, compact, onOpen } = props;
  return (
    <RecordCard
      head={head}
      title={title}
      clamp={1}
      time={time}
      compact={compact}
      openLabel={`Open ${label}`}
      onOpen={onOpen}
      attrs={{ 'data-record': label }}
      signals={(
        <Flex gap="xs" wrap align="center">
          {chips.map((chip) => <Tag key={chip.id} variant="normal" color="neutral" title={chip.title}>{chip.text}</Tag>)}
        </Flex>
      )}
      foot={job ? <Text variant="caption" tone={job.state === 'failed' ? 'danger' : 'muted'}>{jobText(job)}</Text> : undefined}
    />
  );
};

export { WorktreeCard };
