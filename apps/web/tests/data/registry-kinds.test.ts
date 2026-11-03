/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { CODE_TABLE, ITEM_REGISTRY, REGISTRY_KINDS, kindByCode, kindOf } from '../../src/data/registry';
import type { RegistryKind } from '../../src/data/registry';

const KNOWN_TABS = [
  'overview',
  'children',
  'cases',
  'run',
  'links',
  'discussion',
  'tests',
  'media',
  'builds',
  'activity',
  'work',
  'result',
  'runs',
  'log',
  'checks',
  'steps',
  'files',
  'transcript',
  'trail',
];

const CATEGORY_KINDS = ['epic', 'story', 'task', 'bug', 'debt', 'test-plan', 'test-case', 'build', 'worktree', 'agent'];

const colorOf = (kind: RegistryKind): string | undefined => kind.look.color;

describe('I-85 P1 the web reads the codes and looks of every kind of the record registry', { tags: ['critical'] }, () => {
  it('lists the 23 kinds with their code, in the lists of the file', () => {
    const codes = REGISTRY_KINDS.map((kind) => `${kind.id}:${kind.code}`);
    expect(REGISTRY_KINDS).toHaveLength(23);
    expect(codes.slice(0, 9)).toEqual(['epic:E', 'story:S', 'task:K', 'bug:B', 'debt:X', 'test-plan:TP', 'test-case:T', 'build:BL', 'worktree:WT']);
    expect(codes.slice(9, 13)).toEqual(['question:Q', 'decision:D', 'todo:H', 'note:N']);
    expect(codes.slice(13)).toEqual(['agent:A', 'change:PR', 'release:V', 'dependency:DP', 'run:R', 'check:C', 'removal-request:RQ', 'notice:N', 'followup:P', 'message:M']);
  });

  it('gives each of the ten category kinds its own colour and every other kind a tone', () => {
    const colored = REGISTRY_KINDS.filter((kind) => colorOf(kind) !== undefined);
    expect(colored.map((kind) => kind.id).sort()).toEqual([...CATEGORY_KINDS].sort());
    expect(new Set(colored.map(colorOf)).size).toBe(10);
    expect(kindOf('build')?.look.color).toBe('lime');
    expect(kindOf('agent')?.look.color).toBe('pink');
    expect(kindOf('question')?.look.tone).toBe('danger');
    expect(kindOf('release')?.look.tone).toBe('success');
  });

  it('reads the code table longest letters first and leaves the N to the notes', () => {
    const letters = CODE_TABLE.map((entry) => entry.letters);
    expect(letters.indexOf('TP')).toBeLessThan(letters.indexOf('T'));
    expect(letters.indexOf('PR')).toBeLessThan(letters.indexOf('P'));
    expect(letters.indexOf('BL')).toBeLessThan(letters.indexOf('B'));
    expect(letters.filter((entry) => entry === 'N')).toHaveLength(1);
    expect(new Set(letters).size).toBe(letters.length);
    expect(CODE_TABLE).toContainEqual({ letters: 'F', kind: 'followup' });
  });

  it('finds a kind by its letters, an older letter included', () => {
    expect(kindByCode('K')?.id).toBe('task');
    expect(kindByCode('TP')?.id).toBe('test-plan');
    expect(kindByCode('T')?.id).toBe('test-case');
    expect(kindByCode('N')?.id).toBe('note');
    expect(kindByCode('F')?.id).toBe('followup');
    expect(kindByCode('I')).toBeUndefined();
    expect(kindByCode('ZZ')).toBeUndefined();
  });

  it('shows a dependency as DP and stores it as D', () => {
    const dep = kindOf('dependency');
    expect(dep?.code).toBe('DP');
    expect(dep?.stored).toBe('D');
    expect(dep?.collection).toBe('deps');
  });

  it('shows a removal request as RQ and stores it as R, leaving the bare R to the runs', () => {
    const request = kindOf('removal-request');
    expect(request?.code).toBe('RQ');
    expect(request?.stored).toBe('R');
    expect(request?.icon).toBe('trash-2');
    expect(request?.collection).toBe('requests');
    expect(request?.tabs).toEqual(['overview', 'links', 'activity']);
    expect(request?.links.map((link) => link.to)).toEqual([['worktree']]);
    expect(kindByCode('RQ')?.id).toBe('removal-request');
    expect(kindByCode('R')?.id).toBe('run');
  });

});

describe('I-85 P1 the web reads the tabs, links and titles of every kind', { tags: ['critical'] }, () => {
  it('gives every kind tabs from the fixed list, overview first unless it is a log', () => {
    for (const kind of REGISTRY_KINDS) {
      expect(kind.tabs.length, kind.id).toBeGreaterThan(0);
      expect(kind.tabs.filter((tab) => !KNOWN_TABS.includes(tab)), kind.id).toEqual([]);
      if (kind.family !== 'record') expect(kind.tabs[0], kind.id).toBe('overview');
    }
    expect(kindOf('run')?.tabs).toEqual(['log', 'links']);
    expect(kindOf('test-case')?.tabs).toEqual(['overview', 'result', 'links', 'discussion', 'media', 'activity']);
    expect(kindOf('agent')?.tabs).toEqual(['overview', 'work', 'transcript', 'links', 'activity']);
  });

  it('points every link at kinds that exist and groups only by a link it has', () => {
    const ids = new Set(REGISTRY_KINDS.map((kind) => kind.id));
    for (const kind of REGISTRY_KINDS) {
      const unknown = kind.links.flatMap((link) => link.to).filter((target) => !ids.has(target));
      expect(unknown, kind.id).toEqual([]);
      if (kind.groupBy) expect(kind.links.map((link) => link.field), kind.id).toContain(kind.groupBy);
    }
    expect(kindOf('agent')?.groupBy).toBe('items');
    expect(kindOf('bug')?.links.find((link) => link.field === 'duplicateOf')?.inverse).toBe('Duplicates');
  });

  it('keeps the sub-item text field as the title and the records without stages', () => {
    expect(kindOf('question')?.title).toBe('question');
    expect(kindOf('todo')?.collection).toBe('actions');
    expect(ITEM_REGISTRY.records).toHaveLength(10);
    expect(kindOf('change')?.card.progress).toBe('checks');
  });
});
