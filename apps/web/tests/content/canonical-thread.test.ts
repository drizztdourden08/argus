/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { canonicalThread, isLeadRole } from '../../src/data/content';

describe('canonicalThread', () => {
  it('maps the old lead thread ids to the new ones and keeps every other id', () => {
    expect(canonicalThread('coordinator')).toBe('lead');
    expect(canonicalThread('coordinator-hosted')).toBe('lead-hosted');
    expect(canonicalThread('lead')).toBe('lead');
    expect(canonicalThread('worker')).toBe('worker');
  });

  it('knows the lead role', () => {
    expect(isLeadRole('lead')).toBe(true);
    expect(isLeadRole('worker')).toBe(false);
  });
});
