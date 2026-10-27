/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { OwnerFeed } from '../../OwnerFeed';
import type { ActivityTabProps } from './activity.type';

const ActivityTab = (props: ActivityTabProps) => {
  const { boardKey, work, handed } = props;
  if (!work.ready) return <Text tone="muted">Loading the activity.</Text>;
  return <OwnerFeed boardKey={boardKey} work={work} handed={handed} />;
};

export { ActivityTab };
