import { useEffect, useState } from 'react';
import { CollectionEntry } from '../../types';
import { useCommands } from '../../hooks/useCommands';

interface Props {
  activeCreatureId: number | null;
  onSwitched: () => void;
}

export function CreatureManager({ activeCreatureId, onSwitched }: Props) {
  const { getCollection, setActiveCreature } = useCommands();
  const [collection, setCollection] = useState<CollectionEntry[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    getCollection()
      .then(setCollection)
      .catch((err) => {
        console.error('Failed to load collection:', err);
        setError('Failed to load collection.');
      });
  }, [activeCreatureId]);

  const handleSwitch = async (id: number) => {
    await setActiveCreature(id);
    onSwitched();
  };

  return (
    <div className="creature-manager">
      <h2>Collection</h2>
      {error && <p className="error">{error}</p>}
      {!error && collection.length === 0 && <p>No creatures yet. Hatch your first egg!</p>}
      {collection.map(({ creature, species }) => (
        <div key={creature.id} className={`collection-entry ${creature.is_active ? 'active' : ''}`}>
          <pre className="collection-ascii">{species.ascii_idle}</pre>
          <div className="collection-info">
            <div className="collection-name">{creature.nickname ?? species.name}</div>
            <div className="collection-species">{species.name} · {species.rarity_tier}</div>
            <div className="collection-age">Age: {creature.age_ticks} ticks</div>
          </div>
          {!creature.is_active && creature.status === 'alive' && (
            <button onClick={() => handleSwitch(creature.id)}>Set Active</button>
          )}
          {creature.is_active && <span className="active-badge">Active</span>}
        </div>
      ))}
    </div>
  );
}
