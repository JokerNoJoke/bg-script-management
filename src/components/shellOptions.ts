import { store } from "../store";
import type { SelectOption } from "./ui/Select.vue";

/** 脚本表单 / 快速执行共用的 Shell 下拉选项：内置 shell 标注「系统」。 */
export function shellOptions(): SelectOption[] {
  return store.shells.map((s) => ({
    value: s.id,
    label: s.builtin ? `${s.name} · 系统` : s.name,
  }));
}
