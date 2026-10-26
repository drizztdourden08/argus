/* @layer renderer-app @kind component */
import { Card, Flex, Text } from '@drizztdourden08/tessera';
import { NOTIFY_LEVELS, SETTING_PARTS } from '../model';
import type { NotifySettingsProps } from '../notifications.type';
import { SettingToggle } from './SettingToggle';

const NotifySettings = (props: NotifySettingsProps) => {
  const { state } = props;
  const { settings, error, busy, toggle } = state;
  if (!settings) return error ? <Text tone="muted">{error}</Text> : null;
  return (
    <Card title="Notification settings">
      <Flex gap="lg" wrap>
        {NOTIFY_LEVELS.map((level) => (
          <Flex key={level} direction="column" gap="xs" role="group" aria-label={level}>
            <Text variant="overline">{level}</Text>
            {SETTING_PARTS.map((part) => (
              <SettingToggle key={part} part={part} level={level} on={Boolean(settings[part]?.[level])} busy={busy} onToggle={() => toggle(part, level)} />
            ))}
          </Flex>
        ))}
      </Flex>
      {settings.os === false && <Text tone="muted">OS notifications are off in config.json (notify.os): nothing is shown.</Text>}
    </Card>
  );
};

export { NotifySettings };
