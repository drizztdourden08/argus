/* @layer renderer-app @kind component */
import { Button, Callout, Flex, NumberInput, Text } from '@drizztdourden08/tessera';
import { useTitleLimit } from '../behavior/useTitleLimit';
import { TITLE_LIMIT_MAX, TITLE_LIMIT_MIN, TITLE_LIMIT_NOTE } from '../BoardSettings.constants';
import type { SectionProps } from '../BoardSettings.type';

const TitleLimitSection = (props: SectionProps) => {
  const choice = useTitleLimit(props.boardKey);
  return (
    <Flex direction="column" gap="md">
      <Text variant="label">Title length limit</Text>
      <Text variant="caption" tone="muted">{TITLE_LIMIT_NOTE}</Text>
      <Flex gap="md" align="center">
        <NumberInput
          size="sm"
          sizeToContent
          aria-label="Title length limit"
          min={TITLE_LIMIT_MIN}
          max={TITLE_LIMIT_MAX}
          step={1}
          value={choice.value}
          disabled={choice.busy}
          onChange={choice.setValue}
          onKeyDown={(event) => event.key === 'Enter' && void choice.commit()}
        />
        <Button size="sm" variant="primary" disabled={choice.busy || !choice.changed} onClick={() => void choice.commit()}>Save</Button>
      </Flex>
      {choice.problem && <Callout tone="danger">{choice.problem}</Callout>}
    </Flex>
  );
};

export { TitleLimitSection };
