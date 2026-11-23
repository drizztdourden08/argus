/* @layer renderer-app @kind logic */
import type { OverlordSettings, SettingsDraft } from '../../../data/overlord';

const draftOf = (settings: OverlordSettings): SettingsDraft => ({
  cwd: settings.cwd.value,
  exePath: settings.exePath.value,
  budgetUsd: settings.budgetUsd.value ?? Number.NaN,
  restartMax: settings.restartMax.value,
  approvalTimeoutMin: settings.approvalTimeoutMin.value,
});

export { draftOf };
