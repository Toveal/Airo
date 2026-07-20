import { computed, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

/**
 * The single source of truth for monitors and profiles.
 *
 * The backend owns the settings; this is only a mirror of its state. Every
 * profile change is debounced and sent to Rust, which persists it — there is no
 * separate "Save" button because there is nothing left to save by then.
 */

const monitors = ref([]);
const profiles = ref({});
const preferences = ref({
  applyOnClick: true,
  startHidden: false,
});
const autostart = ref(false);
const activeId = ref(null);
/** The monitor the cursor is on right now; delivered as an event from Rust. */
const liveMonitorId = ref(null);
const saveState = ref('idle'); // idle | saving | saved | error
const lastError = ref(null);

let initialized = false;
let savedTimer = null;
const pendingSaves = new Map();

const SAVE_DEBOUNCE_MS = 250;

function flashSaved() {
  saveState.value = 'saved';
  clearTimeout(savedTimer);
  savedTimer = setTimeout(() => {
    if (saveState.value === 'saved') saveState.value = 'idle';
  }, 1600);
}

async function call(command, args) {
  try {
    const result = await invoke(command, args);
    lastError.value = null;
    return result;
  } catch (error) {
    lastError.value = String(error);
    saveState.value = 'error';
    console.error(`[airo] ${command} failed:`, error);
    throw error;
  }
}

async function loadOverview() {
  const overview = await call('get_overview');
  monitors.value = overview.monitors;
  profiles.value = overview.profiles;
  preferences.value = overview.preferences;

  // The active tab may have pointed at a monitor that is now disconnected.
  if (!monitors.value.some((m) => m.id === activeId.value)) {
    activeId.value = monitors.value[0]?.id ?? null;
  }
}

function scheduleSave(monitorId) {
  clearTimeout(pendingSaves.get(monitorId));
  saveState.value = 'saving';

  pendingSaves.set(
    monitorId,
    setTimeout(async () => {
      pendingSaves.delete(monitorId);
      const profile = profiles.value[monitorId];
      if (!profile) return;

      try {
        await call('set_monitor_profile', { monitorId, profile });
        if (pendingSaves.size === 0) flashSaved();
      } catch {
        /* call() has already flagged the error state */
      }
    }, SAVE_DEBOUNCE_MS)
  );
}

export function useAiro() {
  const activeMonitor = computed(
    () => monitors.value.find((m) => m.id === activeId.value) ?? null
  );

  const activeProfile = computed(() =>
    activeId.value ? profiles.value[activeId.value] ?? null : null
  );

  const profileOf = (monitorId) => profiles.value[monitorId] ?? null;

  const setChannel = (monitorId, slot, channel, value) => {
    const profile = profiles.value[monitorId];
    if (!profile) return;
    profile[slot] = { ...profile[slot], [channel]: value };
    scheduleSave(monitorId);
  };

  const setColor = (monitorId, slot, rgb) => {
    const profile = profiles.value[monitorId];
    if (!profile) return;
    profile[slot] = { ...rgb };
    scheduleSave(monitorId);
  };

  const setEnabled = (monitorId, enabled) => {
    const profile = profiles.value[monitorId];
    if (!profile) return;
    profile.enabled = enabled;
    scheduleSave(monitorId);
  };

  const resetMonitor = async (monitorId) => {
    const profile = await call('reset_monitor_profile', { monitorId });
    profiles.value = { ...profiles.value, [monitorId]: profile };
    flashSaved();
  };

  /** Applies the monitor's colours immediately, without waiting for a switch. */
  const previewMonitor = (monitorId) => {
    const profile = profiles.value[monitorId];
    if (!profile) return Promise.resolve(false);
    return call('apply_colors', {
      colors: { highlight: profile.highlight, hotlight: profile.hotlight },
    });
  };

  const restoreColors = () => call('restore_colors');

  const setPreferences = async (patch) => {
    preferences.value = { ...preferences.value, ...patch };
    await call('set_preferences', { preferences: preferences.value });
    flashSaved();
  };

  const setAutostart = async (enabled) => {
    await call('set_autostart', { enabled });
    autostart.value = enabled;
  };

  const refreshMonitors = async () => {
    await call('refresh_monitors');
    await loadOverview();
  };

  /** Idempotent: the state is module level and several components consume it. */
  const init = async () => {
    if (initialized) return;
    initialized = true;

    await loadOverview();

    try {
      autostart.value = await call('get_autostart');
    } catch {
      autostart.value = false;
    }

    await listen('monitors-changed', () => {
      loadOverview().catch(() => {});
    });
    await listen('monitor-changed', (event) => {
      liveMonitorId.value = event.payload.id;
    });
  };

  return {
    monitors,
    profiles,
    preferences,
    autostart,
    activeId,
    activeMonitor,
    activeProfile,
    liveMonitorId,
    saveState,
    lastError,
    init,
    profileOf,
    setChannel,
    setColor,
    setEnabled,
    resetMonitor,
    previewMonitor,
    restoreColors,
    setPreferences,
    setAutostart,
    refreshMonitors,
  };
}
