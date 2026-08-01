<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from "vue";
import type { LogLine } from "../store";

const props = withDefaults(defineProps<{ lines: LogLine[]; paused?: boolean }>(), {
  paused: false,
});

const emit = defineEmits<{ (e: "update:paused", v: boolean): void }>();

// store 单向推入、组件只读；「清空显示」仅用本地 ref 覆盖渲染，不删磁盘日志。
const shown = ref<LogLine[]>([]);
let lastLen = 0;

function sync() {
  const src = props.lines;
  if (src.length < lastLen) {
    // store 超 2000 行做过 splice 截断：全量重算
    shown.value = src.slice();
  } else {
    for (let i = lastLen; i < src.length; i++) shown.value.push(src[i]);
  }
  lastLen = src.length;
}
sync();
watch(() => props.lines.length, sync);

const viewEl = ref<HTMLDivElement | null>(null);
const stick = ref(true);

function scrollToBottom() {
  const el = viewEl.value;
  if (el) el.scrollTop = el.scrollHeight;
}

function onScroll() {
  const el = viewEl.value;
  if (!el) return;
  const dist = el.scrollHeight - el.clientHeight - el.scrollTop;
  stick.value = dist < 60;
}

// 新行到达自动滚到底，除非用户上滚或手动暂停
watch(
  () => shown.value.length,
  () => {
    if (stick.value && !props.paused) void nextTick(scrollToBottom);
  },
);

// 手动恢复时重新跟随到底
watch(
  () => props.paused,
  (p) => {
    if (!p) {
      stick.value = true;
      void nextTick(scrollToBottom);
    }
  },
);

onMounted(() => {
  if (!props.paused) void nextTick(scrollToBottom);
});

function togglePause() {
  emit("update:paused", !props.paused);
}

const copyText = computed(() => shown.value.map((l) => l.text).join("\n"));

async function copy() {
  try {
    await navigator.clipboard.writeText(copyText.value);
  } catch {
    /* 剪贴板不可用时忽略 */
  }
}

function clearView() {
  shown.value = [];
  lastLen = props.lines.length;
}
</script>

<template>
  <div class="log-console">
    <div class="log-toolbar">
      <button
        class="icon-btn"
        :title="paused ? '恢复自动滚动' : '暂停自动滚动'"
        type="button"
        @click="togglePause"
      >
        <svg
          v-if="paused"
          viewBox="0 0 16 16"
          width="13"
          height="13"
          fill="currentColor"
          aria-hidden="true"
        >
          <path d="m5 3 8 5-8 5z" />
        </svg>
        <svg v-else viewBox="0 0 16 16" width="13" height="13" fill="currentColor" aria-hidden="true">
          <rect x="4.5" y="3" width="2.6" height="10" rx="0.8" />
          <rect x="8.9" y="3" width="2.6" height="10" rx="0.8" />
        </svg>
      </button>
      <button class="icon-btn" title="复制全部" type="button" @click="copy">
        <svg
          viewBox="0 0 24 24"
          width="13"
          height="13"
          fill="none"
          stroke="currentColor"
          stroke-width="1.8"
          stroke-linecap="round"
          stroke-linejoin="round"
          aria-hidden="true"
        >
          <rect x="9" y="9" width="11" height="11" rx="2" />
          <path d="M5 15V5a2 2 0 0 1 2-2h10" />
        </svg>
      </button>
      <button class="icon-btn" title="清空显示" type="button" @click="clearView">
        <svg
          viewBox="0 0 24 24"
          width="13"
          height="13"
          fill="none"
          stroke="currentColor"
          stroke-width="1.8"
          stroke-linecap="round"
          stroke-linejoin="round"
          aria-hidden="true"
        >
          <path d="M3 6h18" />
          <path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
          <path d="m19 6-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6" />
        </svg>
      </button>
    </div>
    <div ref="viewEl" class="log-view" @scroll.passive="onScroll">
      <div
        v-for="(line, i) in shown"
        :key="i"
        class="log-line"
        :class="{ 'is-err': line.stream === 'err' }"
      >
        <span class="log-idx">{{ i + 1 }}</span>
        <span class="log-text">{{ line.text }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.log-console {
  display: flex;
  flex-direction: column;
  border-radius: var(--radius-md);
  overflow: hidden;
  background: var(--bg-log);
}

.log-toolbar {
  display: flex;
  justify-content: flex-end;
  gap: 2px;
  padding: 4px 6px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.08);
}

.icon-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border: none;
  border-radius: var(--radius-sm);
  background: transparent;
  color: #8b949e;
  cursor: pointer;
}
.icon-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: #e6e8ec;
}

.log-view {
  max-height: 360px;
  overflow: auto;
  padding: 8px 10px 12px;
  font-family: var(--font-mono);
  font-size: 13px;
  line-height: 1.6;
}

.log-line {
  display: flex;
  gap: 10px;
  color: var(--log-text);
  white-space: pre-wrap;
  word-break: break-all;
}
.log-line.is-err {
  color: var(--log-err);
}

.log-idx {
  flex: none;
  width: 44px;
  text-align: right;
  color: #5b6470;
  user-select: none;
}
.log-text {
  flex: 1;
  min-width: 0;
}
</style>
