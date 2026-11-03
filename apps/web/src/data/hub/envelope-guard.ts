/* @layer renderer-app @kind logic */
import type { HubEnvelope } from './hub-transport.type';

const isRecord = (value: unknown): value is Record<string, unknown> => typeof value === 'object' && value !== null;

const envelopeGuard =
  <M extends { type: string }>(types: readonly string[]) =>
  (data: unknown): data is HubEnvelope<M> => {
    if (!isRecord(data) || typeof data.tab !== 'string' || typeof data.epoch !== 'number') return false;
    const { msg } = data;
    return isRecord(msg) && typeof msg.type === 'string' && types.includes(msg.type);
  };

export { envelopeGuard };
