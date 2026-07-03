import { describe, it, expect } from 'vitest';

describe('stores', () => {
  it('connections store has correct initial state', () => {
    const state = { items: [], activeId: null, loading: false };
    expect(state.items).toEqual([]);
    expect(state.loading).toBe(false);
  });

  it('pull store has correct initial state', () => {
    const state = { jobs: [], unlisten: null };
    expect(state.jobs).toEqual([]);
    expect(state.unlisten).toBeNull();
  });
});
