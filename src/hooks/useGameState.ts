import { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { CreatureState, TickEvent } from '../types';

export function useGameState() {
  const [creatureState, setCreatureState] = useState<CreatureState | null>(null);
  const [eggCount, setEggCount] = useState(0);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    invoke<CreatureState | null>('get_active_creature').then(async (state) => {
      setCreatureState(state);
      if (state) {
        setEggCount(state.egg_count);
      } else {
        const count = await invoke<number>('get_egg_count');
        setEggCount(count);
      }
      setLoading(false);
    });
  }, []);

  useEffect(() => {
    const unlisten = listen<TickEvent>('game-tick', (event) => {
      setCreatureState(event.payload.creature_state);
      setEggCount(event.payload.egg_count);
    });
    return () => { unlisten.then(f => f()); };
  }, []);

  return { creatureState, eggCount, loading, setCreatureState, setEggCount };
}
