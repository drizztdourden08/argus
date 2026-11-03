/* @layer renderer-app @kind types */
import type { ReactNode } from 'react';
import type { ItemRef } from '../../../../data/kanban';
import type { ContentContext } from '../../../../data/content';
import type { ItemField } from '../../../../data/registry';
import type { Option } from '../../../../screens/board/shared';
import type { ItemOps } from '../../../../screens/board/tabs/kanban/kanban.type';

interface FieldGroupModel {
  id: string;
  label: string;
  fields: readonly string[];
}

interface FieldProps {
  name: string;
  item: ItemRef;
  ops: ItemOps;
  cx: ContentContext;
  people: readonly Option[];
}

interface FieldBodyProps {
  spec: ItemField;
  text: string;
  label: string;
}

interface FieldLinesProps {
  label: string;
  lines: readonly string[];
}

interface FieldGroupProps {
  group: FieldGroupModel;
  render: (name: string) => ReactNode;
}

interface FieldFrameProps {
  name: string;
  children: ReactNode;
}

interface FieldSelectProps {
  label: string;
  options: readonly Option[];
  value: string;
  onPick: (value: string) => void;
}

interface RefTagsProps {
  ids: readonly string[];
}

interface FieldViewProps {
  name: string;
  value: unknown;
}

export type { FieldBodyProps, FieldFrameProps, FieldGroupModel, FieldGroupProps, FieldLinesProps, FieldProps, FieldSelectProps, FieldViewProps, RefTagsProps };
