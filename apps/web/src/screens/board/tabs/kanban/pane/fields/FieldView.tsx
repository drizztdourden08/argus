/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { fieldOf } from '../../../../../../data/registry';
import { fieldText } from '../model/field-text';
import type { FieldProps } from '../pane.type';
import { FieldBody } from './FieldBody';
import { NOT_SET } from './FieldView.constants';

const FieldView = (props: FieldProps) => {
  const { name, item, open } = props;
  const spec = fieldOf(name);
  const value = item.fields?.[name];
  if (spec?.content === 'records') return <Text>{`${Array.isArray(value) ? value.length : 0} recorded`}</Text>;
  const text = fieldText(name, value);
  if (!spec || text === '') return <Text tone="muted" italic>{NOT_SET}</Text>;
  return <FieldBody spec={spec} text={text} open={open} />;
};

export { FieldView };
