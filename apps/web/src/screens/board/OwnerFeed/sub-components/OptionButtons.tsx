/* @layer renderer-app @kind component */
import { Button, Flex, Text } from '@drizztdourden08/tessera';
import type { EntryBodyProps } from '../OwnerFeed.type';

const OptionButtons = (props: EntryBodyProps) => {
  const { entry, actions } = props;
  const busy = actions.isBusy(`answer:${entry.id}`);
  return (
    <Flex gap="xs" wrap role="group" aria-label="Options">
      {(entry.options ?? []).map((option, index) => (
        <Button
          key={option}
          size="sm"
          variant="tertiary"
          disabled={busy}
          title={index === 0 ? 'Recommended' : undefined}
          onClick={() => void actions.choose(entry, option)}
        >
          {option}
          {index === 0 ? <> <Text variant="overline" weight={400}>recommended</Text></> : null}
        </Button>
      ))}
    </Flex>
  );
};

export { OptionButtons };
