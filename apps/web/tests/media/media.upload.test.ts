/* @layer renderer-app @kind test */
import { afterEach, describe, expect, it, vi } from 'vitest';
import { uploadItemImage } from '../../src/data/media';

const fetchMock = vi.fn();

vi.stubGlobal('fetch', fetchMock);

afterEach(() => fetchMock.mockReset());

describe('P8 item image upload', { tags: ['critical'] }, () => {
  it('posts the bytes with their own type to the item route and returns the number and url', async () => {
    const body = { item: 'I-7', image: { n: 2, file: 'I-7-2.png', url: '/files/alpha/proofs/I-7-2.png', caption: 'the form' } };
    fetchMock.mockResolvedValueOnce(new Response(JSON.stringify(body), { status: 200 }));
    const file = new File([new Uint8Array([137, 80, 78, 71])], 'shot.png', { type: 'image/png' });
    const result = await uploadItemImage('alpha', 'I-7', file, 'the form');
    const [path, init] = fetchMock.mock.calls[0] as [string, RequestInit];
    expect(path).toBe('/api/p/alpha/items/I-7/images?caption=the%20form');
    expect(init.method).toBe('POST');
    expect((init.headers as Record<string, string>)['Content-Type']).toBe('image/png');
    expect(init.body).toBe(file);
    expect(result.image).toMatchObject({ n: 2, url: '/files/alpha/proofs/I-7-2.png' });
  });

  it('names the picture after the file when no caption is given, and fails with the server error', async () => {
    fetchMock.mockResolvedValueOnce(new Response(JSON.stringify({ error: { code: 'bad_request', message: 'I-7 already has 12 images' } }), { status: 400 }));
    const file = new File(['x'], 'lobby view.png', { type: 'image/png' });
    await expect(uploadItemImage('alpha', 'I-7', file)).rejects.toThrow('I-7 already has 12 images');
    expect((fetchMock.mock.calls[0] as [string])[0]).toBe('/api/p/alpha/items/I-7/images?caption=lobby%20view.png');
  });
});
