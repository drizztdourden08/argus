/* @layer renderer-app @kind test */
import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { PercentField } from '../../src/screens/overview/accounts/sub-components/PercentField';

const mount = (value: number) => {
  const onCommit = vi.fn();
  render(<PercentField label="Auto-switch" value={value} disabled={false} onCommit={onCommit} />);
  return { onCommit, input: screen.getByRole('spinbutton', { name: 'Auto-switch' }) };
};

describe('the percent field of an account rule', { tags: ['critical'] }, () => {
  it('saves a stepper click at once, without a blur', () => {
    const { onCommit } = mount(98);
    fireEvent.click(screen.getByRole('button', { name: 'Increase' }));
    expect(onCommit).toHaveBeenCalledWith(99);
  });

  it('stops the stepper at 100', () => {
    mount(100);
    expect(screen.getByRole('button', { name: 'Increase' })).toHaveProperty('disabled', true);
  });

  it('saves a typed 100 on Enter', () => {
    const { onCommit, input } = mount(98);
    fireEvent.change(input, { target: { value: '100' } });
    fireEvent.keyDown(input, { key: 'Enter' });
    expect(onCommit).toHaveBeenCalledWith(100);
  });

  it('clamps a typed value to 100 on blur', () => {
    const { onCommit, input } = mount(98);
    fireEvent.change(input, { target: { value: '150' } });
    fireEvent.blur(input);
    expect(onCommit).toHaveBeenCalledWith(100);
  });

  it('puts the saved value back when the field is left empty', () => {
    const { onCommit, input } = mount(98);
    fireEvent.change(input, { target: { value: '' } });
    fireEvent.blur(input);
    expect(onCommit).not.toHaveBeenCalled();
    expect(input).toHaveProperty('value', '98');
  });
});
