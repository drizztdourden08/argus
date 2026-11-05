/* @layer renderer-app @kind logic */
import { errorMessage } from '../api';
import type { NotifyLevel } from '../types';
import type { AutoAnswer, AutoPatch, LevelPatch, NotifyState, Refusal } from './notify.type';
import { readAuto } from './read-auto';
import { readSettings } from './read-settings';
import { EMPTY, EMPTY_AUTO } from './settings-store.constants';
import { writeAuto } from './write-auto';
import { writeSettings } from './write-settings';

let state: NotifyState = EMPTY;
let loading: Promise<void> | null = null;
const listeners = new Set<() => void>();

const publish = (next: NotifyState): void => {
  state = next;
  listeners.forEach((listener) => listener());
};

const subscribe = (listener: () => void): (() => void) => {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
};

const getState = (): NotifyState => state;

const withAuto = (answer: AutoAnswer): NotifyState => ({ ...state, os: answer.os, auto: { ...EMPTY_AUTO, ...answer.auto } });

const load = (): Promise<void> => {
  loading ??= Promise.all([readSettings(), readAuto()]).then(
    ([levels, auto]) => publish({ ...withAuto(auto), toast: levels.toast ?? {}, sound: levels.sound ?? {}, ready: true }),
    () => {
      loading = null;
    },
  );
  return loading;
};

const saveLevel = async (level: NotifyLevel, patch: LevelPatch): Promise<Refusal> => {
  const before = state;
  publish({
    ...state,
    toast: patch.toast === undefined ? state.toast : { ...state.toast, [level]: patch.toast },
    sound: patch.sound === undefined ? state.sound : { ...state.sound, [level]: patch.sound },
  });
  try {
    const saved = await writeSettings({
      ...(patch.toast === undefined ? {} : { toast: { [level]: patch.toast } }),
      ...(patch.sound === undefined ? {} : { sound: { [level]: patch.sound } }),
    });
    publish({ ...state, toast: saved.toast ?? state.toast, sound: saved.sound ?? state.sound });
    return null;
  } catch (failure) {
    publish({ ...state, toast: before.toast, sound: before.sound });
    return errorMessage(failure);
  }
};

const saveAuto = async (patch: AutoPatch): Promise<Refusal> => {
  const before = state;
  publish({ ...state, os: patch.os ?? state.os, auto: { ...state.auto, ...patch.auto } });
  try {
    publish(withAuto(await writeAuto(patch)));
    return null;
  } catch (failure) {
    publish({ ...state, os: before.os, auto: before.auto });
    return errorMessage(failure);
  }
};

const resetStore = (): void => {
  loading = null;
  state = EMPTY;
};

const settingsStore = { getState, load, resetStore, saveAuto, saveLevel, subscribe };

export { settingsStore };
