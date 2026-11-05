/* @layer renderer-app @kind test */
import { TesseraProvider } from '@drizztdourden08/tessera';
import { cleanup, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { OWNER_DEFAULTS } from '../../src/owner';
import { OwnerMark } from '../../src/shell/OwnerMark';

afterEach(cleanup);

const mount = (avatar: string | null, name = 'Ann') => {
  vi.stubGlobal('CSS', { supports: () => false });
  return render(
    <TesseraProvider overrides={{}}>
      <OwnerMark profile={{ ...OWNER_DEFAULTS, name, avatar }} />
    </TesseraProvider>,
  );
};

describe('I-111 the owner mark', { tags: ['critical'] }, () => {
  it('labels every face with the name', () => {
    mount(null);
    expect(screen.getByLabelText('Ann')).toBeTruthy();
  });

  it('draws the image face as a picture', () => {
    const { container } = mount('data:image/png;base64,AAAA');
    expect(container.querySelector('img')?.getAttribute('src')).toBe('data:image/png;base64,AAAA');
  });

  it('draws the initials face with the initials', () => {
    const { container } = mount('initials');
    expect(container.textContent).toContain('ME');
  });

  it('draws the brand face as the app icon', () => {
    const { container } = mount('brand:argus');
    expect(container.querySelector('svg, img')).not.toBeNull();
  });
});
