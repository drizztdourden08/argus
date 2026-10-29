/* @layer renderer-app @kind component */
import { Tag } from '@drizztdourden08/tessera';
import { REPLAY_LOOK } from './ReplayTag.constants';
import type { ReplayTagProps } from './ReplayTag.type';

const ReplayTag = (props: ReplayTagProps) => {
  const { replay } = props;
  if (!replay) return null;
  const look = REPLAY_LOOK[replay.state] ?? REPLAY_LOOK.pending;
  const title = replay.at ? `${look.title} (${replay.at})` : look.title;
  if (replay.state === 'done' && replay.url) {
    return <Tag variant="urgency" color={look.color} title={title} href={replay.url} target="_blank" rel="noreferrer">{look.label}</Tag>;
  }
  return <Tag variant="urgency" color={look.color} title={title}>{look.label}</Tag>;
};

export { ReplayTag };
