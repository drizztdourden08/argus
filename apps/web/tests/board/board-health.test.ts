/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { healthOf } from '../../src/screens/board/BoardHeader/behavior/board-health';
import type { HealthInput } from '../../src/screens/board/BoardHeader/BoardHeader.type';

const NOW = Date.parse('2026-11-22T12:00:00Z');

const input = (over: Partial<HealthInput> & { status?: string; activity?: string }): HealthInput => {
  const { status = 'active', activity, ...rest } = over;
  return {
    lead: { id: 'lead', name: 'lead', role: 'coordinator', status, activity },
    faults: 0,
    offline: false,
    updatedAt: '2026-11-22T11:59:00Z',
    now: NOW,
    ...rest,
  };
};

describe('H1 the board health mark', { tags: ['critical'] }, () => {
  it('is green for an active lead on a fresh board', () => {
    expect(healthOf(input({})).tone).toBe('success');
  });

  it('is amber when the lead is paused or stalled, or the board is stale', () => {
    expect(healthOf(input({ status: 'paused' })).tone).toBe('warning');
    expect(healthOf(input({ activity: 'stalled' })).tone).toBe('warning');
    expect(healthOf(input({ updatedAt: '2026-11-22T09:00:00Z' })).tone).toBe('warning');
  });

  it('is red with faults, a gone lead, no lead or no connection, and says why', () => {
    expect(healthOf(input({ faults: 2 })).reason).toBe('2 registered threads are in fault.');
    expect(healthOf(input({ status: 'gone' })).tone).toBe('danger');
    expect(healthOf(input({ lead: null })).tone).toBe('danger');
    expect(healthOf(input({ offline: true })).tone).toBe('danger');
  });
});
