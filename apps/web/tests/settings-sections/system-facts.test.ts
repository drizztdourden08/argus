/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { systemFacts } from '../../src/shell/AppSettings/sections/SystemSection/system-facts';

const NOW = Date.parse('2026-11-04T12:00:00Z');
const labelled = (facts: { label: string; value: string }[]) => Object.fromEntries(facts.map((f) => [f.label, f.value]));

describe('I-111 the system facts of the settings dialog', { tags: ['critical'] }, () => {
  it('maps health, version and remote to the six rows', () => {
    const facts = labelled(
      systemFacts(
        {
          health: { startedAt: '2026-11-04T10:00:00Z', port: 47615 },
          version: { version: '0.42.0', schema: 18 },
          remote: { mode: 'local', replay: {}, syncs: { argus: { at: '2026-11-03T15:00:00Z', by: null, note: '' } } },
        },
        NOW,
      ),
    );
    expect(facts).toEqual({
      Version: '0.42.0 (schema 18)',
      Started: '2h ago',
      Port: '47615',
      Mode: 'live',
      Remote: 'local',
      'Last sync': 'argus 21h ago',
    });
  });

  it('names shadow and preview and falls back to unknown or never', () => {
    expect(labelled(systemFacts({ health: { shadow: true }, version: null, remote: null }, NOW)).Mode).toBe('shadow');
    expect(labelled(systemFacts({ health: { preview: true }, version: null, remote: null }, NOW)).Mode).toBe('preview');
    const empty = labelled(systemFacts({ health: null, version: null, remote: { mode: 'remote', replay: {} } }, NOW));
    expect(empty).toMatchObject({ Version: 'unknown', Started: 'unknown', Port: 'unknown', Remote: 'remote', 'Last sync': 'never' });
  });

  it('shows the newest sync of several', () => {
    const syncs = { a: { at: '2026-11-04T11:00:00Z', by: null, note: '' }, b: { at: '2026-11-04T11:59:00Z', by: null, note: '' } };
    const facts = labelled(systemFacts({ health: null, version: null, remote: { mode: 'local', replay: {}, syncs } }, NOW));
    expect(facts['Last sync']).toBe('b 1m ago');
  });
});
