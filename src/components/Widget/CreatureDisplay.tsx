import { useEffect, useRef, useState } from 'react';
import { CreatureState } from '../../types';

interface Props {
  state: CreatureState;
}

function getFrames(state: CreatureState): string[] {
  const { species, stats } = state;
  if (stats.hunger <= 30) return [species.ascii_hungry];
  if (stats.happiness >= 80 && species.ascii_special) {
    return [species.ascii_happy, species.ascii_special];
  }
  return [species.ascii_idle, species.ascii_happy];
}

export function CreatureDisplay({ state }: Props) {
  const [frameIdx, setFrameIdx] = useState(0);
  const frames = getFrames(state);
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  useEffect(() => {
    if (intervalRef.current) clearInterval(intervalRef.current);
    setFrameIdx(0);
    intervalRef.current = setInterval(() => {
      setFrameIdx(i => (i + 1) % frames.length);
    }, 500);
    return () => { if (intervalRef.current) clearInterval(intervalRef.current); };
  }, [state.creature.id, state.stats.hunger, state.stats.happiness]);

  const name = state.creature.nickname ?? state.species.name;

  return (
    <div className="creature-display" data-tauri-drag-region>
      <div className="creature-name" data-tauri-drag-region>{name}</div>
      <pre className="creature-ascii" data-tauri-drag-region>{frames[frameIdx]}</pre>
      <div className="creature-age" data-tauri-drag-region>Age: {state.creature.age_ticks} ticks</div>
    </div>
  );
}
