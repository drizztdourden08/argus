/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import type { AccountsSettings, HandoverRule } from '../../src/data/types';
import { cleanDraft } from '../../src/shell/AppSettings/sub-components/AccountsSection/behavior/clean-draft';

const settings = (switchAt: number, handover: HandoverRule): AccountsSettings => ({
  pollMinutes: 5, fastPollSeconds: 60, fastAt: 80, forceQuit: false, warnAt: 80, proposeAt: 95, archiveRule: 'newest',
  estimateDefaults: { opus: 0.3, sonnet: 0.1, haiku: 0.03, lead: 0.4, longContextFactor: 1.5, safety: 0.5 }, k: 1,
  accounts: [{ key: 'hotmail', label: 'Hotmail', color: 'blue', cliDir: 'C:/x/hotmail', canReceive: true, rules: { autoSwitch: true, switchAt, handover } }],
});

const rulesOf = (draft: AccountsSettings) => draft.accounts[0]?.rules;

describe('the accounts draft before a save', { tags: ['critical'] }, () => {
  it('keeps 100 for the switch and the handover', () => {
    expect(rulesOf(cleanDraft(settings(100, 100), settings(98, 'auto')))).toMatchObject({ switchAt: 100, handover: 100 });
  });

  it('clamps a percent above 100 or under 1', () => {
    expect(rulesOf(cleanDraft(settings(150, 0), null))).toMatchObject({ switchAt: 100, handover: 1 });
  });

  it('falls back to the saved value when a field was left empty', () => {
    expect(rulesOf(cleanDraft(settings(Number.NaN, Number.NaN), settings(97, 96)))).toMatchObject({ switchAt: 97, handover: 96 });
  });

  it('leaves off and auto alone', () => {
    expect(rulesOf(cleanDraft(settings(98, 'off'), null))?.handover).toBe('off');
  });
});
