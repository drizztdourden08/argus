/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import type { AccountsSwitch } from '../../src/data/types';
import { canSkip, wizardSteps } from '../../src/screens/overview/accounts/switch/model';

const ids = (doc: AccountsSwitch) => wizardSteps(doc).steps.map((step) => step.id);

const ruleFlow: AccountsSwitch = {
  phase: 'handover',
  steps: [
    { stage: 'detect', state: 'done', text: 'Found.' },
    { stage: 'handover', state: 'running', text: 'Asked.' },
  ],
  threads: [{ board: 'rotp', thread: 'lead', state: 'asked' }],
};

describe('the wizard steps of an account switch', { tags: ['critical'] }, () => {
  it('puts the handover before the countdown in the rule flow', () => {
    expect(ids(ruleFlow)).toEqual(['detect', 'handover', 'countdown', 'sync', 'switch', 'verify']);
  });

  it('puts the countdown first when it opens the flow', () => {
    const doc: AccountsSwitch = { phase: 'countdown', steps: [{ stage: 'detect', state: 'done' }, { stage: 'countdown', state: 'running' }] };
    expect(ids(doc)).toEqual(['detect', 'countdown', 'handover', 'sync', 'switch', 'verify']);
  });

  it('points the current id at the running stage', () => {
    expect(wizardSteps(ruleFlow).currentId).toBe('handover');
  });

  it('points the current id at the first stage not done when nothing runs', () => {
    const doc: AccountsSwitch = { steps: [{ stage: 'detect', state: 'done' }, { stage: 'handover', state: 'done' }] };
    expect(wizardSteps(doc).currentId).toBe('countdown');
  });

  it('is complete only when the switch is done', () => {
    expect(wizardSteps(ruleFlow).complete).toBe(false);
    expect(wizardSteps({ phase: 'done' }).complete).toBe(true);
  });

  it('strikes a skipped stage', () => {
    const doc: AccountsSwitch = { phase: 'sync', steps: [{ stage: 'countdown', state: 'skipped' }] };
    expect(wizardSteps(doc).steps.find((step) => step.id === 'countdown')?.struck).toBe(true);
  });

  it('marks a failed stage with its error text', () => {
    const doc: AccountsSwitch = { phase: 'failed', steps: [{ stage: 'sync', state: 'failed' }], sync: { state: 'failed', error: 'Disk is full.' } };
    const step = wizardSteps(doc).steps.find((candidate) => candidate.id === 'sync');
    expect(step?.error).toBe(true);
    expect(step?.summary).toBe('Disk is full.');
  });

  it('lets the owner skip the countdown, the handover and the check, never the sync', () => {
    expect(canSkip({ phase: 'handover' }, 'handover')).toBe(true);
    expect(canSkip({ phase: 'countdown' }, 'countdown')).toBe(true);
    expect(canSkip({ phase: 'verifying' }, 'verify')).toBe(true);
    expect(canSkip({ phase: 'sync' }, 'sync')).toBe(false);
    expect(canSkip({ phase: 'handover' }, 'countdown')).toBe(false);
  });
});
