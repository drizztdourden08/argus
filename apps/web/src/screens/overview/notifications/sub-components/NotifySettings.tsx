/* @layer renderer-app @kind component */
import { Inline, Text } from '@drizztdourden08/tessera';
import { NOTIFY_LEVELS, SETTING_PARTS } from '../model';
import type { NotifySettingsProps } from '../notifications.type';
import { SettingToggle } from './SettingToggle';

const NotifySettings = (props: NotifySettingsProps) => {
  const { state } = props;
  const { settings, error, busy, toggle } = state;
  if (!settings) return error ? <Text tone="muted">{error}</Text> : null;
  return (
    <Inline gap="lg" align="center" wrap role="group" aria-label="Notification settings">
      <Text variant="subtitle" weight={600}>Notification settings</Text>
      {NOTIFY_LEVELS.map((level) => (
        <Inline key={level} gap="xs" align="center" role="group" aria-label={level}>
          <Text variant="overline">{level}</Text>
          {SETTING_PARTS.map((part) => (
            <SettingToggle key={part} part={part} level={level} on={Boolean(settings[part]?.[level])} busy={busy} onToggle={() => toggle(part, level)} />
          ))}
        </Inline>
      ))}
      {settings.os === false && <Text tone="muted">OS notifications are off in config.json (notify.os): nothing is shown.</Text>}
    </Inline>
  );
};

export { NotifySettings };
