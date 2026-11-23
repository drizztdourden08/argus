/* @layer renderer-app @kind logic */
import type { OverlordSettings, OverlordSettingsPatch, SettingsDraft } from '../../../data/overlord';

const draftOf = (settings: OverlordSettings): SettingsDraft => ({
  cwd: settings.cwd.value,
  claudeExe: settings.claudeExe.value,
  budgetUsd: settings.budgetUsd.value ?? Number.NaN,
  restartMax: settings.restartMax.value,
  approvalTimeoutMin: settings.approvalTimeoutMin.value,
});

const wireNumber = (value: number): number | '' => (Number.isFinite(value) ? value : '');

/** Only the fields the owner changed; a blank goes as an empty value, which resets it. */
const patchOf = (settings: OverlordSettings, draft: SettingsDraft): OverlordSettingsPatch => {
  const before = draftOf(settings);
  const patch: OverlordSettingsPatch = {};
  if (draft.cwd.trim() !== before.cwd) patch.cwd = draft.cwd.trim();
  if (draft.claudeExe.trim() !== before.claudeExe) patch.claudeExe = draft.claudeExe.trim();
  if (!Object.is(draft.budgetUsd, before.budgetUsd)) patch.budgetUsd = wireNumber(draft.budgetUsd);
  if (!Object.is(draft.restartMax, before.restartMax)) patch.restartMax = wireNumber(draft.restartMax);
  if (!Object.is(draft.approvalTimeoutMin, before.approvalTimeoutMin)) {
    patch.approvalTimeoutMin = wireNumber(draft.approvalTimeoutMin);
  }
  return patch;
};

export { draftOf, patchOf };
