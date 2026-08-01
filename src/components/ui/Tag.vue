<script setup lang="ts">
withDefaults(defineProps<{ tone?: string; label?: string; dot?: boolean }>(), {
  tone: "neutral",
  label: "",
  dot: true,
});
</script>

<template>
  <span class="tag" :class="`tag-${tone}`">
    <span v-if="dot" class="tag-dot" aria-hidden="true" />
    <slot>{{ label }}</slot>
  </span>
</template>

<style scoped>
.tag {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 1px 8px;
  border-radius: 999px;
  font-size: var(--font-xs);
  line-height: 18px;
  color: var(--tag-color);
  background: color-mix(in srgb, var(--tag-color) 12%, transparent);
  white-space: nowrap;
}
.tag-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--tag-color);
}
.tag-running {
  --tag-color: var(--status-running);
}
.tag-success {
  --tag-color: var(--status-success);
}
.tag-failed {
  --tag-color: var(--status-failed);
}
.tag-timeout {
  --tag-color: var(--status-timeout);
}
.tag-killed {
  --tag-color: var(--status-killed);
}
.tag-interrupted {
  --tag-color: var(--status-interrupted);
}
.tag-neutral {
  --tag-color: var(--text-tertiary);
}
</style>
