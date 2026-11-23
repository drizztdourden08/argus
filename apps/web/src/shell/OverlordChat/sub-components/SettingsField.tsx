/* @layer renderer-app @kind component */
import { Button, Field, Flex, Stack, Text } from '@drizztdourden08/tessera';
import { RESET_LABEL, SETTINGS_LABELS, SOURCE_CAPTIONS } from '../OverlordChat.constants';
import type { SettingsFieldProps } from '../behavior/chat-settings.type';

const SettingsField = (props: SettingsFieldProps) => {
  const { name, source, busy, onReset, children } = props;
  return (
    <Stack gap="xs">
      <Field label={SETTINGS_LABELS[name]}>{children}</Field>
      <Flex gap="sm" align="center">
        <Text variant="caption" tone="muted">{SOURCE_CAPTIONS[source]}</Text>
        <Button disabled={busy || source !== 'set'} onClick={() => onReset(name)}>{RESET_LABEL}</Button>
      </Flex>
    </Stack>
  );
};

export { SettingsField };
