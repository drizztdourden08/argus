/* @layer renderer-app @kind component */
import { Checkbox, Stack, Text } from '@drizztdourden08/tessera';
import { archivedNote, candidateKey } from '../model';
import { CLEANUP_NONE } from '../review.constants';
import type { CleanupListProps } from '../review-props.type';

const CleanupList = (props: CleanupListProps) => {
  const { accounts, found, picked, now, onToggle } = props;
  if (found.candidates.length === 0) return <Text tone="muted">{CLEANUP_NONE}</Text>;
  return (
    <Stack gap="sm">
      {accounts.map((account) => {
        const mine = found.candidates.filter((candidate) => candidate.account === account.key);
        if (mine.length === 0) return null;
        return (
          <Stack key={account.key} gap="xs">
            <Text variant="overline">{`${account.label ?? account.key} (${mine.length})`}</Text>
            {mine.map((candidate) => {
              const key = candidateKey(candidate);
              const age = archivedNote({ archived: true, lastActivityAt: candidate.lastActivityAt }, now);
              return (
                <Checkbox
                  key={key}
                  size="sm"
                  checked={picked.has(key)}
                  onChange={() => onToggle(key)}
                  label={`${candidate.title ?? candidate.sessionId}${candidate.folder ? ` (${candidate.folder})` : ''}${age ? `, ${age}` : ''}`}
                />
              );
            })}
          </Stack>
        );
      })}
    </Stack>
  );
};

export { CleanupList };
