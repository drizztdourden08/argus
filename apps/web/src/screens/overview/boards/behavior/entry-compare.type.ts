/* @layer renderer-app @kind types */
import type { BoardEntry } from '../boards-view.type';

type EntryCompare = (left: BoardEntry, right: BoardEntry) => number;

export type { EntryCompare };
