/* @layer renderer-app @kind component */
import { Flex, Stack, Text } from '@drizztdourden08/tessera';
import { usageSections } from '../behavior/usage-sections';
import type { UsagePanelProps } from '../behavior/chat-settings.type';

const UsagePanel = (props: UsagePanelProps) => {
  const { usage } = props;
  return (
    <Stack gap="md">
      {usageSections(usage).map((section) => (
        <Stack key={section.title} gap="2xs">
          <Text variant="overline" tone="muted">{section.title}</Text>
          {section.rows.map((row) => (
            <Flex key={row.label} justify="between" gap="md">
              <Text variant="caption" tone="muted">{row.label}</Text>
              <Text variant="caption" numeric>{row.value}</Text>
            </Flex>
          ))}
        </Stack>
      ))}
    </Stack>
  );
};

export { UsagePanel };
