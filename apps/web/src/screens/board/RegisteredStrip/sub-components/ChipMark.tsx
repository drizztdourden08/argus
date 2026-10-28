/* @layer renderer-app @kind component */
import { Badge, Icon, Spinner, Text } from '@drizztdourden08/tessera';
import type { ChipMarkProps } from '../RegisteredStrip.type';

const ChipMark = (props: ChipMarkProps) => {
  const { look, name } = props;
  const word = <Text as="span" weight={600}>{name}</Text>;
  if (look.mark === 'spin') {
    return (
      <>
        <Spinner size="sm" label="Wake on its way" />
        {word}
      </>
    );
  }
  if (look.mark === 'warn') {
    return (
      <>
        <Text as="span" tone={look.tone === 'danger' ? 'danger' : 'warning'}><Icon name="triangle-alert" size={12} aria-hidden /></Text>
        {word}
      </>
    );
  }
  return (
    <>
      <Badge variant="dot" color={look.tone === 'neutral' ? 'tame' : look.tone} />
      {word}
    </>
  );
};

export { ChipMark };
