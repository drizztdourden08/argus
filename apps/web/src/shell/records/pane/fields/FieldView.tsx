/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { fieldOf } from '../../../../data/registry';
import { fieldLabel } from '../model/field-label';
import { fieldText } from '../model/field-text';
import { FieldBody } from './FieldBody';
import type { FieldViewProps } from './fields.type';
import { NOT_SET } from './FieldView.constants';

const FieldView = (props: FieldViewProps) => {
  const { name, value } = props;
  const spec = fieldOf(name);
  if (spec?.content === 'records') return <Text>{`${Array.isArray(value) ? value.length : 0} recorded`}</Text>;
  const text = fieldText(name, value);
  if (!spec || text === '') return <Text tone="muted" italic>{NOT_SET}</Text>;
  return <FieldBody spec={spec} text={text} label={fieldLabel(name)} />;
};

export { FieldView };
