import { type ComputedRef, watch } from "vue";
import { spin } from "@/util/spin";
import type { SpinInstance } from "@/util/types";

/**
 * 选曲后歌词/谱面未就绪前显示全屏 loading；就绪后关闭。
 */
export const use_selection_spin = (spinning: ComputedRef<boolean>) => {
  let spin_instance: SpinInstance | null = null;

  watch(
    () => spinning.value,
    (new_val, old_val) => {
      if (new_val === old_val) return;
      if (new_val) {
        spin_instance = spin();
      } else {
        spin_instance?.close();
        spin_instance = null;
      }
    },
  );
};
