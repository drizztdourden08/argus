/* @layer renderer-app @kind logic */
import type { OverlordSettings, OverlordSettingsPatch, SettingsDraft } from '../../../data/overlord';
import { draftOf } from './draft-of';

const wireNumber = (value: number): number | '' => (Number.isFinite(value) ? value : '');

const patchOf = (settings: OverlordSettings, draft: SettingsDraft): OverlordSettingsPatch => {
  const before = draftOf(settings);
  const patch: OverlordSettingsPatch = {};
  if (draft.cwd.trim() !== before.cwd) patch.cwd = draft.cwd.trim();
  if (draft.exePath.trim() !== before.exePath) patch.exePath = draft.exePath.trim();
  if (!Object.is(draft.budgetUsd, before.budgetUsd)) patch.budgetUsd = wireNumber(draft.budgetUsd);
  if (!Object.is(draft.restartMax, before.restartMax)) patch.restartMax = wireNumber(draft.restartMax);
  if (!Object.is(draft.approvalTimeoutMin, before.approvalTimeoutMin)) {
    patch.approvalTimeoutMin = wireNumber(draft.approvalTimeoutMin);
  }
  return patch;
};

export { patchOf };
