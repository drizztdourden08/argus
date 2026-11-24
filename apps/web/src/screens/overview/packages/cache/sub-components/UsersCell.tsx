/* @layer renderer-app @kind component */
import { FoldText, Text } from '@drizztdourden08/tessera';
import { useNow } from '../../../clock';
import { USERS_LINES } from '../cache.constants';
import type { UsersCellProps } from '../cache.type';
import { unusedNote } from '../model';

const UsersCell = (props: UsersCellProps) => {
  const { row } = props;
  const now = useNow();
  if (row.userCount === 0) return <Text size="md" tone="muted">{unusedNote(row.item, now)}</Text>;
  return <FoldText lines={USERS_LINES} fill>{row.users}</FoldText>;
};

export { UsersCell };
