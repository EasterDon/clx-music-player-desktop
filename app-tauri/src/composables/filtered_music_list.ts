import { computed, ref } from "vue";
import type { ChangeEvent } from "ant-design-vue/es/_util/EventInterface";
import { use_music_store } from "@/store/player";

/**
 * 曲库展示列表：筛选条件为空时与 store 全量一致；有输入时按曲名包含匹配（不区分大小写）。
 */
export const use_filtered_music_list = () => {
  const music_store = use_music_store();
  const filter_text = ref("");

  const music_list_display = computed(() => {
    const text = filter_text.value.trim().toLowerCase();
    if (!text) {
      return [...music_store.music_list];
    }
    return music_store.music_list.filter(
      (music) => music.name.toLowerCase().indexOf(text) > -1,
    );
  });

  const on_music_filter = (e: ChangeEvent) => {
    filter_text.value = e.target.value?.trim() ?? "";
  };

  return { music_list_display, on_music_filter, filter_text };
};
