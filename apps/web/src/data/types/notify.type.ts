/* @layer renderer-app @kind types */
type NotifyLevel = 'info' | 'ping' | 'urgent';

type LevelSwitches = Partial<Record<NotifyLevel, boolean>>;

interface NotifySettings {
  os?: boolean;
  toast?: LevelSwitches;
  sound?: LevelSwitches;
}

interface TestPingResult {
  recorded?: boolean;
  id?: string;
  n?: number;
  level?: string;
  toast?: string;
  why?: string | null;
}

export type { LevelSwitches, NotifyLevel, NotifySettings, TestPingResult };
