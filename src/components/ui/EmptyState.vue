<script setup lang="ts">
withDefaults(defineProps<{ title: string; description?: string }>(), {
  description: "",
});
</script>

<template>
  <div class="empty">
    <svg
      class="empty-icon"
      viewBox="0 0 24 24"
      width="44"
      height="44"
      fill="none"
      stroke="currentColor"
      stroke-width="1.5"
      stroke-linecap="round"
      stroke-linejoin="round"
      aria-hidden="true"
    >
      <rect x="3" y="4" width="18" height="16" rx="2" />
      <path d="M3 9h18" />
      <path d="m9 14 2 2 4-4" />
    </svg>
    <p class="empty-title">{{ title }}</p>
    <p v-if="description" class="empty-desc">{{ description }}</p>
    <div v-if="$slots.default" class="empty-actions">
      <slot />
    </div>
  </div>
</template>

<style scoped>
.empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--space-2);
  padding: var(--space-10) var(--space-6);
  text-align: center;
}
.empty-icon {
  color: var(--text-tertiary);
  opacity: 0.7;
}
.empty-title {
  font-size: var(--font-md);
  font-weight: 500;
  color: var(--text-primary);
}
.empty-desc {
  font-size: var(--font-sm);
  color: var(--text-tertiary);
  max-width: 360px;
}
.empty-actions {
  margin-top: var(--space-2);
  display: flex;
  gap: var(--space-2);
}
</style>
