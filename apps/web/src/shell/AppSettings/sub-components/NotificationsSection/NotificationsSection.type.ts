/* @layer renderer-app @kind types */
import type { AutoLevels } from '../../../../data/notify';
import type { NotifyLevel } from '../../../../data/types';

type Report = (id: string, refusal: string | null) => void;

interface LevelLabel {
  level: NotifyLevel;
  title: string;
}

interface AutoLabel {
  key: keyof AutoLevels;
  title: string;
}

interface TestNotification {
  line: string;
  busy: boolean;
  send: () => void;
}

export type { AutoLabel, LevelLabel, Report, TestNotification };
