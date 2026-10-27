/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import type { Agent } from '../../src/data/agents';
import type { WithId } from '../../src/data/react';
import type { RunEntry } from '../../src/data/runs';
import {
  agentElapsed, agentMessage, agentNow, agentProblem, clipText, findLead, finishedCount, formatDuration, isQuiet, isStale, itemOptions, MODEL_LOOKS, modelFamily, modelTip,
  plainText, runSpan, shortBranch, sortActive, sortDone, statusLook, tagLook, tokenLines,
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
    expect(statusLook('stop-requested')).toEqual({ label: 'stopping', tone: 'warning' });
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

  it('checks an agent field against its limit, and a task against empty', () => {
    expect(agentProblem('task', '  ')).toBe('A task cannot be empty.');
    expect(agentProblem('note', '')).toBeNull();
    expect(agentProblem('note', 'x'.repeat(101))).toBe('At most 100 characters.');
    expect(agentProblem('branch', 'x'.repeat(80))).toBeNull();
  });

  it('lists every live item with a short title, the current one even when gone, and none first', () => {
    const items = [{ id: 'I-1', title: 'a'.repeat(60), status: 'todo', thread: 't' }, { id: 'I-2', title: 'gone', status: 'todo', thread: 't', deletedAt: 'x' }];
    const options = itemOptions(items, 'I-7');
    expect(options.map((option) => option.value)).toEqual(['', 'I-1', 'I-7']);
    expect(options[1]?.label).toBe(`I-1 ${'a'.repeat(40)}`);
  });

  it('tones a model by family: sonnet green, haiku blue, opus and others neutral', () => {
    expect(MODEL_LOOKS.sonnet.tone).toBe('success');
    expect(MODEL_LOOKS.haiku.tone).toBe('info');
    expect(MODEL_LOOKS.opus).toEqual({ tone: 'neutral', dot: true });
    expect(tagLook('warning')).toEqual({ variant: 'urgency', color: 'warning' });
    expect(tagLook('neutral')).toEqual({ variant: 'normal', color: 'neutral' });
  });

  it('writes a run span as 1m 05s and 2h 03m', () => {
    const run = (start: string, end: string) => ({ id: 'R-1', name: 'x', status: 'exited', queuedAt: start, startedAt: start, endedAt: end, exitCode: 0, error: null, lastLine: null }) as const;
    expect(runSpan(run('2026-01-01T00:00:00Z', '2026-01-01T00:01:05Z'))).toBe('1m 05s');
    expect(runSpan(run('2026-01-01T00:00:00Z', '2026-01-01T02:03:00Z'))).toBe('2h 03m');
    expect(runSpan(run('2026-01-01T00:00:00Z', '2026-01-01T00:00:09Z'))).toBe('9s');
  });

  it('takes the live coordinator as the lead and ignores merged and deleted ones', () => {
    const thread = (id: string, extra: Record<string, unknown> = {}) => ({ id, name: id, role: 'coordinator', status: 'active', ...extra });
    expect(findLead([thread('old', { status: 'done' }), thread('now')])?.id).toBe('now');
    expect(findLead([thread('old', { status: 'done' })])?.id).toBe('old');
    expect(findLead([thread('gone', { deletedAt: 'x' }), thread('merged', { mergedInto: 'y' })])).toBeNull();
  });
});

describe('finishedCount', () => {
  const agent = (id: string): WithId<Agent> => ({ id, status: 'done' });
  const run = (id: string, status: string): RunEntry => ({ id, status, queuedAt: at(MIN) }) as RunEntry;

  it('adds the finished runs to the finished agents, as the Node Show finished button does', () => {
    const agents = [agent('A-1'), agent('A-2')];
    const runs = [run('R-1', 'exited'), run('R-2', 'failed'), run('R-3', 'running')];
    expect(finishedCount(agents, runs)).toBe(4);
    expect(finishedCount(agents, [])).toBe(2);
  });
});
