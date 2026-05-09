import { CreatureState } from '../../types';

interface Props {
  state: CreatureState;
  onFeed: () => void;
  onPlay: () => void;
  onHatch: () => void;
  onToggleExpand: () => void;
  expanded: boolean;
  disabled: boolean;
}

export function ActionButtons({ state, onFeed, onPlay, onHatch, onToggleExpand, expanded, disabled }: Props) {
  return (
    <div className="action-buttons">
      <button onClick={onFeed} disabled={disabled || state.stats.hunger >= 100}>
        Feed
      </button>
      <button onClick={onPlay} disabled={disabled || state.stats.energy < 10}>
        Play
      </button>
      {state.egg_count > 0 && (
        <button onClick={onHatch} disabled={disabled} className="hatch-btn">
          🥚 ({state.egg_count})
        </button>
      )}
      <button onClick={onToggleExpand} className="expand-btn" title={expanded ? 'Collapse' : 'Expand'}>
        {expanded ? '▲' : '⋯'}
      </button>
    </div>
  );
}
