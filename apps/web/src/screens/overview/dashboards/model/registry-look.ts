/* @layer renderer-app @kind logic */
import { formatAgo } from '../../format';
import { isMainRow } from './is-main-row';
import { registryTip } from './registry-tip';
import type { ChipLook, RegistryItem } from './model.type';

const faultLook = (row: RegistryItem, now: number): ChipLook => ({
  label: row.fault?.reason === 'board-closed' ? 'board closed' : 'wake failed',
  tone: 'danger',
  pulse: false,
  tip: row.fault ? registryTip(row.fault, isMainRow(row), now) : '',
});

const wakingLook = (row: RegistryItem, name: string, now: number): ChipLook => {
  const since = row.pendingWake?.since ?? row.since ?? null;
  const ago = formatAgo(since, now);
  return { label: 'waking', since, tone: 'info', pulse: true, tip: `${name}: waking up, a wake was sent ${ago} ago and has not been answered yet` };
};

const busyLook = (name: string): ChipLook => ({
  label: 'working',
  tone: 'success',
  pulse: false,
  tip: `${name}: in a turn right now; its wake listener starts again at its next tool call or when the turn ends`,
});

const listeningLook = (row: RegistryItem, name: string, now: number): ChipLook => {
  const label = row.listenerLabel ? ` (${row.listenerLabel})` : '';
  const since = row.since ? ` since ${formatAgo(row.since, now)} ago` : '';
  return { label: '', tone: 'success', pulse: false, tip: `${name}: listening${label}${since}` };
};

const idleLook = (row: RegistryItem, name: string): ChipLook => ({
  label: 'not listening',
  tone: row.deaf ? 'warning' : 'neutral',
  pulse: false,
  tip: `${name}: not listening, so the dashboard cannot wake it${row.deaf ? '. The session is active: the Overlord is told when this lasts.' : ''}`,
});

const registryLook = (row: RegistryItem, name: string, now: number): ChipLook => {
  if (row.state === 'fault' && row.fault) return faultLook(row, now);
  if (row.state === 'waking') return wakingLook(row, name, now);
  if (row.state === 'busy') return busyLook(name);
  if (row.state === 'listening') return listeningLook(row, name, now);
  return idleLook(row, name);
};

export { registryLook };
