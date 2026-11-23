/* @layer renderer-app @kind component */
import { Button, Inline, Stack, Text } from '@drizztdourden08/tessera';
import { COPY_TITLE, MERGE_LABEL, MERGE_TITLE } from '../compare.constants';
import type { SyncChoicesProps } from '../cards.type';
import { copyLabel, copyPairs } from '../model';

const SyncChoices = (props: SyncChoicesProps) => {
  const { accounts, disabled, actions } = props;
  return (
    <Stack gap="xs">
      <Text variant="overline" tone="muted">{COPY_TITLE}</Text>
      <Inline gap="xs" wrap>
        {copyPairs(accounts).map((pair) => (
          <Button key={`${pair.from.id}>${pair.to.id}`} size="xs" variant="primary" disabled={disabled} onClick={() => actions.copy(pair.from.key ?? pair.from.id, pair.to.key ?? pair.to.id)}>
            {copyLabel(accounts, pair)}
          </Button>
        ))}
      </Inline>
      <Text variant="overline" tone="muted">{MERGE_TITLE}</Text>
      <Inline gap="xs" wrap>
        <Button size="xs" variant="secondary" disabled={disabled} onClick={actions.merge}>{MERGE_LABEL}</Button>
      </Inline>
    </Stack>
  );
};

export { SyncChoices };
