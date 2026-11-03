/* @layer renderer-app @kind logic */
import { kindOf } from '../../../../data/registry';
import type { RegistryKind } from '../../../../data/registry';
import type { RecordRef } from '../../records.type';
import { NAMING_FIELDS } from './pane-model.constants';
import type { PaneDoc } from '../pane.type';

const kindFor = (target: RecordRef, doc: PaneDoc | null): RegistryKind | undefined => {
  const field = NAMING_FIELDS[target.collection];
  const named = field && doc ? doc[field] : undefined;
  return (typeof named === 'string' ? kindOf(named) : undefined) ?? kindOf(target.kind);
};

export { kindFor };
