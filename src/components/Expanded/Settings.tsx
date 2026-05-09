import { useEffect, useState } from 'react';
import { Settings as SettingsType } from '../../types';
import { useCommands } from '../../hooks/useCommands';

interface Props {
  onChanged: (settings: SettingsType) => void;
}

export function Settings({ onChanged }: Props) {
  const { getSettings, updateSettings } = useCommands();
  const [settings, setSettings] = useState<SettingsType | null>(null);

  useEffect(() => {
    getSettings().then(setSettings);
  }, []);

  if (!settings) return <div>Loading settings...</div>;

  const patch = async (changes: Partial<SettingsType>) => {
    const updated = await updateSettings(changes);
    setSettings(updated);
    onChanged(updated);
  };

  return (
    <div className="settings">
      <h2>Settings</h2>

      <label className="setting-row">
        <span>Always on top</span>
        <input
          type="checkbox"
          checked={settings.always_on_top}
          onChange={e => patch({ always_on_top: e.target.checked })}
        />
      </label>

      <label className="setting-row">
        <span>Show tray tooltip</span>
        <input
          type="checkbox"
          checked={settings.show_tray_tooltip}
          onChange={e => patch({ show_tray_tooltip: e.target.checked })}
        />
      </label>

      <label className="setting-row">
        <span>Tick interval (seconds)</span>
        <input
          type="number"
          min={10}
          max={3600}
          value={settings.tick_interval_secs}
          onChange={e => patch({ tick_interval_secs: parseInt(e.target.value, 10) })}
        />
      </label>
    </div>
  );
}
