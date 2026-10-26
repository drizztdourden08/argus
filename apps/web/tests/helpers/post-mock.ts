/* @layer renderer-app @kind test */
import { waitFor } from '@testing-library/react';
import { expect, vi } from 'vitest';

interface SentPost {
  path: string;
  body: Record<string, unknown>;
}

const reply = (status: string, outcome: string | null) =>
  new Response(JSON.stringify({ id: 'O-test', status, outcome, entry: null, seq: 12 }));

const mockPosts = (status = 'applied', outcome: string | null = null) => {
  const fetchMock = vi.fn<typeof fetch>(() => Promise.resolve(reply(status, outcome)));
  vi.stubGlobal('fetch', fetchMock);
  return fetchMock;
};

const sentPosts = (fetchMock: ReturnType<typeof mockPosts>): SentPost[] =>
  fetchMock.mock.calls.map(([input, init]) => ({ path: String(input), body: JSON.parse(String(init?.body)) as Record<string, unknown> }));

const onlyPost = async (fetchMock: ReturnType<typeof mockPosts>): Promise<SentPost | undefined> => {
  await waitFor(() => expect(sentPosts(fetchMock)).toHaveLength(1));
  const [post] = sentPosts(fetchMock);
  expect(post?.path).toBe('/api/p/rotp/entries');
  return post;
};

export { mockPosts, onlyPost, sentPosts };
