/* @layer renderer-app @kind test */
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { accountsSettingsStore } from '../../src/data/accounts/accounts-settings-store';

const { changeAccount, getState, load, resetStore, save } = accountsSettingsStore;

const fetchMock = vi.fn();
vi.stubGlobal('fetch', fetchMock);

const account = (switchAt: number, handover: number | 'auto' = 'auto') => ({
  key: 'hotmail', label: 'Hotmail', color: '#6f9cf0', cliDir: 'C:/x/hotmail', canReceive: true,
  rules: { autoSwitch: true, switchAt, handover },
});

const settings = (switchAt: number, handover: number | 'auto' = 'auto') => ({ pollMinutes: 5, accounts: [account(switchAt, handover)] });

const json = (body: unknown) => new Response(JSON.stringify(body), { status: 200 });

const sentBody = (call: number): { accounts: ReturnType<typeof account>[] } => {
  const [, init] = fetchMock.mock.calls[call] as [string, RequestInit];
  return JSON.parse(String(init.body)) as { accounts: ReturnType<typeof account>[] };
};

beforeEach(() => resetStore());

afterEach(() => fetchMock.mockReset());

describe('the shared accounts settings store', { tags: ['critical'] }, () => {
  it('builds a rule change on the last saved settings, not a stale copy', async () => {
    fetchMock.mockResolvedValueOnce(json(settings(98)));
    await load();
    fetchMock.mockResolvedValueOnce(json(settings(100)));
    await save({ accounts: [account(100)] });
    fetchMock.mockResolvedValueOnce(json(settings(100, 99)));
    await changeAccount('hotmail', { rules: { handover: 99 } });
    expect(sentBody(2).accounts[0]?.rules).toEqual({ autoSwitch: true, switchAt: 100, handover: 99 });
    expect(getState().settings?.accounts[0]?.rules.switchAt).toBe(100);
  });

  it('keeps a save when an older read lands after it', async () => {
    let answerRead: (response: Response) => void = () => undefined;
    fetchMock.mockReturnValueOnce(new Promise<Response>((resolve) => {
      answerRead = resolve;
    }));
    const reading = load();
    fetchMock.mockResolvedValueOnce(json(settings(100)));
    await save({ accounts: [account(100)] });
    answerRead(json(settings(98)));
    await reading;
    expect(getState().settings?.accounts[0]?.rules.switchAt).toBe(100);
  });

  it('is busy only while a save is out', async () => {
    fetchMock.mockResolvedValueOnce(json(settings(98)));
    await load();
    fetchMock.mockResolvedValueOnce(json(settings(99)));
    const saving = changeAccount('hotmail', { rules: { switchAt: 99 } });
    expect(getState().busy).toBe(true);
    await saving;
    expect(getState().busy).toBe(false);
  });
});
