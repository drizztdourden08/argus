/* @layer renderer-app @kind component */
import { cloneElement } from 'react';
import { FIT_FIELD, FIT_SEARCH } from '../shared.constants';
import type { FieldFitProps } from '../shared.type';
import './FieldFit.css';

const FieldFit = (props: FieldFitProps) => {
  const { search = false, children } = props;
  return cloneElement(children, { className: search ? FIT_SEARCH : FIT_FIELD });
};

export { FieldFit };
