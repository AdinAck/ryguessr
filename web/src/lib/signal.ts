import { useEffect, useLayoutEffect, useRef } from "react";

import PlayerLeave from "@/types/player-leave";
import PlayerData from "@/types/player-data";

export function createSignal<T = void>() {
  const listeners = new Set<(payload: T) => void>();
  return {
    emit: (payload?: T) => listeners.forEach((l) => l(payload as T)),

    subscribe: (l: (payload: T) => void) => {
      listeners.add(l);
      return () => listeners.delete(l);
    },
  };
}

export function useSignal<T>(
  signal: { subscribe: (l: (payload: T) => void) => () => void },
  handler: (payload: T) => void,
) {
  const ref = useRef(handler);

  useLayoutEffect(() => {
    ref.current = handler;
  });

  useEffect(() => {
    return signal.subscribe((payload) => ref.current(payload));
  }, [signal]);
}

export const playerJoinSignal = createSignal<PlayerData>();
export const playerLeaveSignal = createSignal<PlayerLeave>();
export const refreshSseEventStream = createSignal();
