/* @layer renderer-app @kind component */
import { Disclosure, SectionHeader, Stack } from '@drizztdourden08/tessera';
import type { BoardsGroupProps } from '../boards-view.type';

const BoardsGroup = (props: BoardsGroupProps) => {
  const { group, children } = props;
  const { label, entries, folded } = group;
  if (folded) return <Disclosure summary={`${label} (${entries.length})`}>{children}</Disclosure>;
  return (
    <Stack gap="sm">
      <SectionHeader title={label} count={entries.length} level={2} />
      {children}
    </Stack>
  );
};

export { BoardsGroup };
