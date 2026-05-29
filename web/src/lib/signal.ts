import { useEffect, useRef } from "react";

export function createSignal() {
  const listeners = new Set<() => void>();
  return {
    emit: () => listeners.forEach((l) => l()),
    subscribe: (l: () => void) => {
      listeners.add(l);
      return () => listeners.delete(l);
    },
  };
}

export function useSignal(
  signal: { subscribe: (l: () => void) => () => void },
  handler: () => void,
) {
  const ref = useRef(handler);
  ref.current = handler;
  useEffect(() => signal.subscribe(() => ref.current()), [signal]);
}

export const refreshSseEventStream = createSignal();
