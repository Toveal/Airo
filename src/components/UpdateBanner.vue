<template>
  <!-- The bottom-most strip of the window: it only exists while there is
       something to say about an update, so it is absent from the layout the
       rest of the time. -->
  <Transition name="banner">
    <div v-if="visible" class="update-banner" :class="state">
      <div class="update-info">
        <span class="update-icon" aria-hidden="true">
          <svg viewBox="0 0 16 16" width="15" height="15">
            <path d="M8 10.5V2M8 2 4.5 5.5M8 2l3.5 3.5M2.5 12.5h11" fill="none"
              stroke="currentColor" stroke-width="1.4" stroke-linecap="round"
              stroke-linejoin="round" />
          </svg>
        </span>

        <div class="update-text">
          <template v-if="state === 'available'">
            <span class="update-title">{{ t('update.available', { version: info?.version }) }}</span>
            <span v-if="info?.notes" class="update-notes" :title="info.notes">{{ info.notes }}</span>
          </template>
          <template v-else-if="state === 'downloading'">
            <span class="update-title">{{ t('update.downloading', { percent }) }}</span>
            <span class="update-bar"><span class="update-bar-fill" :style="{ width: percent + '%' }" /></span>
          </template>
          <span v-else-if="state === 'installing'" class="update-title">{{ t('update.installing') }}</span>
          <span v-else-if="state === 'error'" class="update-title update-error-text">{{ t('update.error') }}</span>
        </div>
      </div>

      <div class="update-actions">
        <template v-if="state === 'available'">
          <button class="ubtn ubtn-ghost" @click="dismiss">{{ t('update.later') }}</button>
          <button class="ubtn ubtn-primary" @click="downloadAndInstall">{{ t('update.download') }}</button>
        </template>
        <button v-else-if="state === 'error'" class="ubtn ubtn-ghost" @click="dismiss">
          {{ t('update.later') }}
        </button>
      </div>
    </div>
  </Transition>
</template>

<script setup>
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { useUpdater } from '../composables/useUpdater.js';

const { t } = useI18n();
const { state, info, progress, dismissed, downloadAndInstall, dismiss } = useUpdater();

const visible = computed(() => state.value !== 'idle' && !dismissed.value);

const percent = computed(() => {
  const { downloaded, total } = progress.value;
  if (!total) return 0;
  return Math.min(100, Math.round((downloaded / total) * 100));
});
</script>

<style scoped>
.update-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 24px;
  border-top: 1px solid var(--border-primary);
  background: var(--bg-secondary);
  flex-shrink: 0;
}

.update-banner.error {
  background: color-mix(in srgb, #e05252 12%, var(--bg-secondary));
}

.update-info {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
}

.update-icon {
  display: inline-flex;
  color: var(--accent);
  flex-shrink: 0;
}

.error .update-icon {
  color: #e05252;
}

.update-text {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.update-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
}

.update-error-text {
  color: #e05252;
  font-weight: 500;
}

.update-notes {
  font-size: 12px;
  color: var(--text-tertiary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.update-bar {
  display: block;
  width: 200px;
  max-width: 40vw;
  height: 4px;
  margin-top: 2px;
  border-radius: 2px;
  background: var(--bg-tertiary);
  overflow: hidden;
}

.update-bar-fill {
  display: block;
  height: 100%;
  background: var(--accent);
  border-radius: 2px;
  transition: width var(--transition-fast);
}

.update-actions {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}

.ubtn {
  display: inline-flex;
  align-items: center;
  padding: 7px 14px;
  border: none;
  border-radius: var(--radius-md);
  font-family: 'Outfit', sans-serif;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  white-space: nowrap;
  transition: all var(--transition-fast);
}

.ubtn-ghost {
  background: transparent;
  color: var(--text-tertiary);
}

.ubtn-ghost:hover {
  background: var(--bg-hover);
  color: var(--text-secondary);
}

.ubtn-primary {
  background: var(--accent);
  color: #ffffff;
  font-weight: 600;
}

.ubtn-primary:hover {
  background: var(--accent-hover);
  box-shadow: 0 2px 12px var(--accent-glow);
}

.banner-enter-active,
.banner-leave-active {
  transition: opacity var(--transition-fast), transform var(--transition-fast);
}

.banner-enter-from,
.banner-leave-to {
  opacity: 0;
  transform: translateY(100%);
}

@media (max-width: 560px) {
  .update-banner {
    padding-left: 14px;
    padding-right: 14px;
  }

  .update-notes {
    display: none;
  }
}
</style>
