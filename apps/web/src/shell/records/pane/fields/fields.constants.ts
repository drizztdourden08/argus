/* @layer renderer-app @kind constants */
import type { ComponentType } from 'react';
import type { ContentKind } from '../../../../data/registry';
import type { FieldProps } from './fields.type';
import { EnumField } from './EnumField';
import { LineField } from './LineField';
import { PersonField } from './PersonField';
import { RichField } from './RichField';

const FIELD_EDITORS: Readonly<Partial<Record<ContentKind, ComponentType<FieldProps>>>> = {
  line: LineField,
  slug: LineField,
  date: LineField,
  ref: LineField,
  refs: LineField,
  rich: RichField,
  enum: EnumField,
  person: PersonField,
};

export { FIELD_EDITORS };
