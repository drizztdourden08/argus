/* @layer renderer-app @kind logic */
import type { NotifyLevel } from '../../../../data/types';
import { NOTIFY_LEVELS } from './model.constants';
import type { NoteItem } from './model.type';

const noteLevel = (note: NoteItem): NotifyLevel => NOTIFY_LEVELS.find((level) => level === note.level) ?? 'ping';

export { noteLevel };
