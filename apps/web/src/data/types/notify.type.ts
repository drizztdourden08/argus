/* @layer renderer-app @kind types */
type NotifyLevel = 'info' | 'ping' | 'urgent';

interface NotificationDoc {
  n?: number;
  at?: string;
  level?: string;
  title?: string;
  text?: string;
  project?: string | null;
  thread?: string | null;
  by?: string;
  toast?: string;
  url?: string | null;
  why?: string | null;
}

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

export type { LevelSwitches, NotificationDoc, NotifyLevel, NotifySettings, TestPingResult };
