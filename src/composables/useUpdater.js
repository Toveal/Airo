import { ref } from 'vue';
import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

/**
 * Update checking against the GitHub release feed.
 *
 * Runs once on startup: if a newer release exists, a banner offers to install
 * it. Everything is module level so the banner and any other consumer share
 * one state — the same shape as `useAiro`.
 *
 * The `Update` handle from `check()` is not reactive and cannot go through
 * Vue's proxy without losing its methods, so it is kept in a plain module
 * variable and only its data is mirrored into refs.
 */

// idle → nothing to show. available → banner with buttons.
// downloading/installing → progress. error → install failed.
const state = ref('idle'); // idle | available | downloading | installing | error
const info = ref(null); // { version, notes }
const progress = ref({ downloaded: 0, total: 0 });
const dismissed = ref(false); // "Later": hidden until the next launch

let pending = null; // the Update handle, valid while state === 'available'
let checked = false;

async function checkForUpdates() {
  if (checked) return;
  checked = true;

  try {
    const update = await check();
    // `check()` returns null when the app is already up to date.
    if (!update) return;

    pending = update;
    info.value = { version: update.version, notes: update.body ?? '' };
    state.value = 'available';
  } catch (error) {
    // Offline, no release yet, bad manifest — none of these should disrupt
    // the app. Stay silent and let the user keep working.
    console.error('[airo] update check failed:', error);
  }
}

async function downloadAndInstall() {
  if (!pending) return;

  state.value = 'downloading';
  progress.value = { downloaded: 0, total: 0 };

  try {
    await pending.downloadAndInstall((event) => {
      switch (event.event) {
        case 'Started':
          progress.value = { downloaded: 0, total: event.data.contentLength ?? 0 };
          break;
        case 'Progress':
          progress.value = {
            downloaded: progress.value.downloaded + event.data.chunkLength,
            total: progress.value.total,
          };
          break;
        case 'Finished':
          state.value = 'installing';
          break;
      }
    });

    // On Windows the passive installer usually terminates the app itself, so
    // this may never be reached — that is fine. Guarded in case it is.
    await relaunch();
  } catch (error) {
    console.error('[airo] update install failed:', error);
    state.value = 'error';
  }
}

function dismiss() {
  dismissed.value = true;
}

export function useUpdater() {
  return {
    state,
    info,
    progress,
    dismissed,
    checkForUpdates,
    downloadAndInstall,
    dismiss,
  };
}
