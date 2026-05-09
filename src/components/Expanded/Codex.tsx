import { useEffect, useState } from 'react';
import { CodexEntry, RarityTier } from '../../types';
import { useCommands } from '../../hooks/useCommands';

const RARITY_COLORS: Record<RarityTier, string> = {
  common: '#aaa',
  uncommon: '#4af',
  rare: '#a4f',
  legendary: '#fa4',
};

function silhouette(ascii: string): string {
  return ascii.replace(/[^\n]/g, '░');
}

export function Codex() {
  const { getCodex } = useCommands();
  const [entries, setEntries] = useState<CodexEntry[]>([]);

  useEffect(() => {
    getCodex().then(setEntries);
  }, []);

  return (
    <div className="codex">
      <h2>Codex</h2>
      <div className="codex-grid">
        {entries.map(entry => (
          <div key={entry.species.id} className="codex-card">
            <pre className="codex-ascii">
              {entry.discovered ? entry.species.ascii_idle : silhouette(entry.species.ascii_idle)}
            </pre>
            <div className="codex-name">
              {entry.discovered ? entry.species.name : '???'}
            </div>
            <div className="codex-tier" style={{ color: RARITY_COLORS[entry.species.rarity_tier] }}>
              {entry.species.rarity_tier}
            </div>
            {entry.discovered && (
              <div className="codex-detail">
                <div>{entry.species.codex_flavor_text}</div>
                <div>Hatched {entry.times_hatched}×</div>
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
