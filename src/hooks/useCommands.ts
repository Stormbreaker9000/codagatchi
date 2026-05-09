import { invoke } from '@tauri-apps/api/core';
import {
  CreatureState, CreatureStats, CodexEntry,
  Settings, SettingsPatch, CollectionEntry
} from '../types';

export function useCommands() {
  const feed = (creatureId: number) =>
    invoke<CreatureStats>('feed', { creature_id: creatureId });

  const play = (creatureId: number) =>
    invoke<CreatureStats>('play', { creature_id: creatureId });

  const hatchEgg = () =>
    invoke<CreatureState>('hatch_egg');

  const setActiveCreature = (creatureId: number) =>
    invoke<void>('set_active_creature', { creature_id: creatureId });

  const getCodex = () =>
    invoke<CodexEntry[]>('get_codex');

  const getSettings = () =>
    invoke<Settings>('get_settings');

  const updateSettings = (patch: SettingsPatch) =>
    invoke<Settings>('update_settings', { patch });

  const getCollection = () =>
    invoke<CollectionEntry[]>('get_collection');

  return { feed, play, hatchEgg, setActiveCreature, getCodex, getSettings, updateSettings, getCollection };
}
