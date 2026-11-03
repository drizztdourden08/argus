/* @layer renderer-app @kind logic */
import type { ItemRef } from '../../../../../../data/kanban';
import { firstStageIn } from '../../../../../../data/registry';
import { STATUS_OPTIONS, typeOf } from '../../model';
import type { StageFact } from '../pane.type';
import { paneType } from './pane-type';

const stageFact = (item: ItemRef): StageFact => {
  const type = paneType(item);
  if (!type || !(item.type || item.stage)) return { label: 'Status', options: STATUS_OPTIONS, value: item.status, target: (status) => ({ status }) };
  const current = type.stages.find((stage) => stage.id === item.stage) ?? firstStageIn(typeOf(item), item.status);
  const wanted = new Set([current?.id, ...(current?.next ?? type.stages.map((stage) => stage.id))]);
  const options = type.stages.filter((stage) => wanted.has(stage.id)).map((stage) => ({ value: stage.id, label: stage.label }));
  return { label: 'Stage', options, value: current?.id ?? item.status, target: (stage) => ({ stage }) };
};

export { stageFact };
