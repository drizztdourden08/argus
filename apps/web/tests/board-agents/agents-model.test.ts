/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import {
  agentElapsed, agentMessage, agentNow, clipText, filterAgents, findLead, formatDuration, isQuiet, isStale, modelFamily, modelTip,
  plainText, shortBranch, sortActive, sortDone, statusLook, tokenLines,
} from '../../src/screens/board/tabs/agents/model';

const NOW = Date.parse('2026-10-26T12:00:00Z');
const MIN = 60_000;
const at = (msAgo: number): string => new Date(NOW - msAgo).toISOString();

describe('formatDuration', () => {
  it('writes minutes, hours and days the way the Node page does', () => {
    expect(formatDuration(30_000)).toBe('<1m');
    expect(formatDuration(5 * MIN)).toBe('5m');
    expect(formatDuration(60 * MIN)).toBe('1h');
    expect(formatDuration(135 * MIN)).toBe('2h 15m');
    expect(formatDuration(49 * 60 * MIN)).toBe('2d');
    expect(formatDuration(-5)).toBe('<1m');
  });
});

describe('agent clocks', () => {
  it('counts a running agent up to now and a finished one to its end', () => {
    expect(agentElapsed({ status: 'working', startedAt: at(12 * MIN) }, NOW)).toBe('12m');
    expect(agentElapsed({ status: 'done', startedAt: at(30 * MIN), finishedAt: at(10 * MIN) }, NOW)).toBe('20m');
    expect(agentElapsed({ status: 'done' }, NOW)).toBe('-');
  });

  it('turns stale after 45 minutes without an update, only while working', () => {
    expect(isStale({ status: 'working', updatedAt: at(46 * MIN) }, NOW)).toBe(true);
    expect(isStale({ status: 'working', updatedAt: at(44 * MIN) }, NOW)).toBe(false);
    expect(isStale({ status: 'waiting', updatedAt: at(90 * MIN) }, NOW)).toBe(false);
  });

  it('calls a working agent quiet when it is stalled or between tools, never without live data', () => {
    expect(isQuiet({ status: 'working' })).toBe(false);
    expect(isQuiet({ status: 'working', live: { activity: 'tool' } })).toBe(false);
    expect(isQuiet({ status: 'working', live: { activity: 'waiting' } })).toBe(true);
    expect(isQuiet({ status: 'working', live: { activity: 'tool', stalled: true } })).toBe(true);
  });
});

describe('agentNow and agentMessage', () => {
  it('names the tool with the file name for file tools and the summary for the others', () => {
    const tool = (name: string, summary: string) => ({ status: 'working', live: { activity: 'tool', activitySince: at(1000), lastTool: { name, summary } } });
    expect(agentNow(tool('Edit', 'X:\\repo\\src\\a.ts'))?.text).toBe('Edit a.ts');
    expect(agentNow(tool('Bash', 'npm test'))?.text).toBe('Bash npm test');
    expect(agentNow({ status: 'working', live: { activity: 'tool' } })?.text).toBe('tool');
  });

  it('gives waiting no age, a finished agent no Now, and falls back to the live note', () => {
    expect(agentNow({ status: 'working', live: { activity: 'waiting', activitySince: at(1000) } })).toEqual({ text: 'waiting', since: null });
    expect(agentNow({ status: 'done', live: { activity: 'tool' } })).toBeNull();
    expect(agentNow({ status: 'working', live: { liveNote: 'no transcript found' } })?.text).toBe('no transcript found');
    expect(agentNow({ status: 'working' })).toBeNull();
  });

  it('prefers the transcript message and falls back to the result of a finished agent only', () => {
    expect(agentMessage({ status: 'done', result: 'r', live: { lastMessage: 'm' } })).toBe('m');
    expect(agentMessage({ status: 'done', result: 'r' })).toBe('r');
    expect(agentMessage({ status: 'working', result: 'r' })).toBe('');
  });
});

describe('text helpers', () => {
  it('strips the marks of a message and keeps its words on one line', () => {
    expect(plainText('# Title\n\nFixed **bold**, _soft_ and `code`.\n\n- one\n- two\n\n[a link](http://x.test)')).toBe('Title Fixed bold, soft and code. one two a link');
    expect(plainText('```ts\nconst a = 1;\n```')).toBe('const a = 1;');
  });

  it('clips a long text with three dots and leaves a short one alone', () => {
    expect(clipText('short', 10)).toBe('short');
    expect(clipText('a long sentence here', 6)).toBe('a long...');
  });

  it('shortens a branch to what follows the first slash', () => {
    expect(shortBranch('agent/lead-x')).toBe('lead-x');
    expect(shortBranch('main')).toBe('main');
  });
});

describe('model and status', () => {
  it('reads the family of a model name', () => {
    expect(modelFamily('sonnet')).toBe('sonnet');
    expect(modelFamily('opus-5-5')).toBe('opus');
    expect(modelFamily('fable-9')).toBe('other');
  });

  it('puts the why and the model it runs on in the tip', () => {
    expect(modelTip({ status: 'working', modelWhy: 'known cause' })).toBe('known cause');
    expect(modelTip({ status: 'working' })).toBe('no justification recorded');
    expect(modelTip({ status: 'working', modelWhy: 'x', live: { modelUsed: 'sonnet-5-5' } })).toBe('x (running on sonnet-5-5)');
  });

  it('lists the counters and the token line of the live view', () => {
    expect(tokenLines({ status: 'working' })).toEqual([]);
    expect(tokenLines({ status: 'working', live: { turns: 7, toolCalls: 12, modelUsed: 'm', tokens: { input: 12_340, output: 2100, cacheRead: 98_000 } } })).toEqual([
      'Turns 7, tool calls 12',
      'Tokens: 12,340 in, 2,100 out, 98,000 cache read',
      'Running on m',
    ]);
  });

  it('calls the stop request stopping and keeps an unknown status as it is', () => {
    expect(statusLook('stop-requested').label).toBe('stopping');
    expect(statusLook('failed').tone).toBe('danger');
    expect(statusLook('odd')).toEqual({ label: 'odd', tone: 'neutral' });
  });
});

describe('lists', () => {
  const doc = (id: string, extra: Record<string, unknown> = {}) => ({ id, status: 'working', ...extra });

  it('sorts working agents by start and then by id, finished ones by end, newest first', () => {
    expect(sortActive([doc('A-10', { startedAt: at(5 * MIN) }), doc('A-2', { startedAt: at(5 * MIN) }), doc('A-3', { startedAt: at(9 * MIN) })]).map((a) => a.id)).toEqual(['A-3', 'A-2', 'A-10']);
    expect(sortDone([doc('A-1', { finishedAt: at(9 * MIN) }), doc('A-2', { finishedAt: at(MIN) })]).map((a) => a.id)).toEqual(['A-2', 'A-1']);
  });

  it('filters by view and by search over id, name, task, item and note', () => {
    const active = [doc('A-1', { name: 'alpha' }), doc('A-2', { name: 'beta', task: 'Port the reaper', item: 'I-9' })];
    const done = [doc('A-3', { status: 'done', note: 'merged' })];
    expect(filterAgents(active, done, 'running', '').map((a) => a.id)).toEqual(['A-1', 'A-2']);
    expect(filterAgents(active, done, 'done', '').map((a) => a.id)).toEqual(['A-3']);
    expect(filterAgents(active, done, 'all', 'REAPER').map((a) => a.id)).toEqual(['A-2']);
    expect(filterAgents(active, done, 'all', 'i-9').map((a) => a.id)).toEqual(['A-2']);
    expect(filterAgents(active, done, 'all', 'merged').map((a) => a.id)).toEqual(['A-3']);
  });

  it('takes the live coordinator as the lead and ignores merged and deleted ones', () => {
    const thread = (id: string, extra: Record<string, unknown> = {}) => ({ id, name: id, role: 'coordinator', status: 'active', ...extra });
    expect(findLead([thread('old', { status: 'done' }), thread('now')])?.id).toBe('now');
    expect(findLead([thread('old', { status: 'done' })])?.id).toBe('old');
    expect(findLead([thread('gone', { deletedAt: 'x' }), thread('merged', { mergedInto: 'y' })])).toBeNull();
  });
});
