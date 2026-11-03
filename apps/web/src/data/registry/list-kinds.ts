/* @layer renderer-app @kind logic */
import type { RegistryKind } from './kind.type';
import type { ItemRegistry, ItemType, RecordType, SubItemType } from './registry.type';

const fromType = (type: ItemType): RegistryKind => ({
  family: 'type',
  id: type.id,
  label: type.label,
  code: type.code,
  aliases: [],
  icon: type.icon,
  look: { color: type.color },
  collection: type.collection,
  idKey: type.idKey,
  view: type.view,
  title: type.title,
  assignee: type.assignee,
  tabs: type.tabs,
  card: type.card,
  links: type.links,
  signals: type.signals ?? [],
  groupBy: type.groupBy,
  action: type.action,
});

const fromSubItem = (sub: SubItemType): RegistryKind => ({
  family: 'subItem',
  id: sub.id,
  label: sub.label,
  code: sub.code,
  aliases: [],
  icon: sub.icon,
  look: { tone: sub.tone },
  collection: sub.store,
  view: sub.view,
  title: sub.field,
  assignee: sub.assignee,
  tabs: sub.tabs,
  card: sub.card,
  links: sub.links,
  signals: sub.signals ?? [],
  groupBy: sub.groupBy,
  action: sub.action,
});

const fromRecord = (record: RecordType): RegistryKind => ({
  family: 'record',
  id: record.id,
  label: record.label,
  code: record.code,
  shares: record.shares,
  aliases: record.aliases ?? [],
  stored: record.stored,
  icon: record.icon,
  look: { color: record.color, tone: record.tone },
  collection: record.collection,
  idKey: record.idKey,
  view: record.view,
  title: record.title,
  assignee: record.assignee,
  tabs: record.tabs,
  card: record.card,
  links: record.links,
  signals: record.signals ?? [],
  groupBy: record.groupBy,
  action: record.action,
});

const listKinds = (registry: ItemRegistry): RegistryKind[] => [
  ...registry.types.map(fromType),
  ...registry.subItems.map(fromSubItem),
  ...registry.records.map(fromRecord),
];

export { listKinds };
