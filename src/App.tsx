import { useState } from 'react';
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';
import { useGameState } from './hooks/useGameState';
import { useCommands } from './hooks/useCommands';
import { CreatureDisplay } from './components/Widget/CreatureDisplay';
import { StatBars } from './components/Widget/StatBars';
import { ActionButtons } from './components/Widget/ActionButtons';
import { Codex } from './components/Expanded/Codex';
import { CreatureManager } from './components/Expanded/CreatureManager';
import { Settings } from './components/Expanded/Settings';
import { Settings as SettingsType } from './types';

const WIDGET = { width: 220, height: 280 };
const EXPANDED = { width: 420, height: 560 };

type Tab = 'codex' | 'collection' | 'settings';

function App() {
  const { creatureState, eggCount, loading, setCreatureState, setEggCount } = useGameState();
  const { feed, play, hatchEgg } = useCommands();
  const [expanded, setExpanded] = useState(false);
  const [tab, setTab] = useState<Tab>('codex');
  const [busy, setBusy] = useState(false);
  const [lastError, setLastError] = useState<string | null>(null);

  const toggleExpand = async () => {
    const next = !expanded;
    const size = next ? EXPANDED : WIDGET;
    try {
      await getCurrentWindow().setSize(new LogicalSize(size.width, size.height));
    } catch (e) {
      console.error('setSize failed:', e);
    }
    setExpanded(next);
  };

  const handleFeed = async () => {
    if (!creatureState) return;
    setBusy(true);
    try {
      const stats = await feed(creatureState.creature.id);
      setCreatureState({ ...creatureState, stats });
    } catch (e) {
      setLastError(`feed: ${e}`);
    } finally {
      setBusy(false);
    }
  };

  const handlePlay = async () => {
    if (!creatureState) return;
    setBusy(true);
    try {
      const stats = await play(creatureState.creature.id);
      setCreatureState({ ...creatureState, stats });
    } catch (e) {
      setLastError(`play: ${e}`);
    } finally {
      setBusy(false);
    }
  };

  const handleHatch = async () => {
    setBusy(true);
    try {
      const newState = await hatchEgg();
      setCreatureState(newState);
      setEggCount(newState.egg_count);
    } catch (e) {
      setLastError(`hatch: ${e}`);
    } finally {
      setBusy(false);
    }
  };

  if (loading) {
    return <div className="loading">Loading...</div>;
  }

  return (
    <div className="app">
      {lastError && (
        <div style={{ background: '#600', color: '#fcc', fontSize: 9, padding: '2px 6px', wordBreak: 'break-all' }}
             onClick={() => setLastError(null)}>
          {lastError}
        </div>
      )}
      <div className="widget-section">
        {creatureState ? (
          <>
            <CreatureDisplay state={creatureState} />
            <StatBars stats={creatureState.stats} />
            <ActionButtons
              state={creatureState}
              onFeed={handleFeed}
              onPlay={handlePlay}
              onHatch={handleHatch}
              onToggleExpand={toggleExpand}
              expanded={expanded}
              disabled={busy}
            />
          </>
        ) : (
          <div className="no-creature">
            <div>No active creature.</div>
            {eggCount > 0 && (
              <button onClick={handleHatch} disabled={busy}>
                🥚 Hatch egg ({eggCount})
              </button>
            )}
          </div>
        )}
      </div>

      {expanded && (
        <div className="expanded-section">
          <div className="tab-bar">
            <button className={`tab ${tab === 'codex' ? 'active' : ''}`} onClick={() => setTab('codex')}>Codex</button>
            <button className={`tab ${tab === 'collection' ? 'active' : ''}`} onClick={() => setTab('collection')}>Collection</button>
            <button className={`tab ${tab === 'settings' ? 'active' : ''}`} onClick={() => setTab('settings')}>Settings</button>
          </div>
          <div className="tab-content">
            {tab === 'codex' && <Codex />}
            {tab === 'collection' && (
              <CreatureManager
                activeCreatureId={creatureState?.creature.id ?? null}
                onSwitched={() => {}}
              />
            )}
            {tab === 'settings' && <Settings onChanged={(_: SettingsType) => {}} />}
          </div>
        </div>
      )}
    </div>
  );
}

export default App;
