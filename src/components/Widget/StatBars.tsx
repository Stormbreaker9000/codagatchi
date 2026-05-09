import { CreatureStats } from '../../types';

interface Props {
  stats: CreatureStats;
}

function StatBar({ label, value, color }: { label: string; value: number; color: string }) {
  return (
    <div className="stat-row">
      <span className="stat-label">{label}</span>
      <div className="stat-bar-track">
        <div
          className="stat-bar-fill"
          style={{ width: `${value}%`, background: color }}
        />
      </div>
      <span className="stat-value">{value}</span>
    </div>
  );
}

export function StatBars({ stats }: Props) {
  return (
    <div className="stat-bars">
      <StatBar label="Hunger"    value={stats.hunger}    color="#e88" />
      <StatBar label="Happiness" value={stats.happiness} color="#8e8" />
      <StatBar label="Energy"    value={stats.energy}    color="#88e" />
    </div>
  );
}
