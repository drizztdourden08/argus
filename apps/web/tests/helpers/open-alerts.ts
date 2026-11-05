/* @layer renderer-app @kind test */
import { fireEvent, screen } from '@testing-library/react';

const openAlerts = async (): Promise<{ button: HTMLElement; panel: HTMLElement }> => {
  const button = await screen.findByRole('button', { name: /^Alerts/ });
  fireEvent.click(button);
  const panel = await screen.findByRole('dialog', { name: 'Alerts' });
  return { button, panel };
};

export { openAlerts };
