import { reactive, ref } from "vue";
import { defineStore } from "pinia";
import { message } from "ant-design-vue";
import { delay, click } from "@/util";

export const use_music_store = defineStore("songs", () => {
  /** 当前选中歌曲信息 */
  const current_music = ref<MusicListItem | null>(null);
  /** 全量替换列表，重复调用不会追加重复项 */
  const set_music_list = (list: MusicListItem[]) => {
    music_list.splice(0, music_list.length, ...list);
  };

  /** 歌曲列表 */
  const music_list = reactive<MusicList>([]);
  const set_current_music = (id: number) => {
    const result = music_list.find((item) => item.id === id);
    current_music.value = result ? result : null;
  };

  /** 是否播放中 */
  const is_playing = ref(false);
  const set_is_playing = (state: boolean) => {
    is_playing.value = state;
  };

  const continue_play = ref(false);
  /** 是否继续播放 */
  const play = async (song_notation: (string | number)[]) => {
    if (!current_music.value) {
      message.warn("请先选择一首歌曲哦");
      return void 0;
    }
    if (continue_play.value) {
      return void 0;
    }

    continue_play.value = true;
    for (const item of song_notation) {
      if (!continue_play.value) {
        break;
      }
      if (typeof item === "number") {
        await delay(item);
      }
      if (typeof item === "string") {
        await click(item);
      }
    }
    continue_play.value = false;
  };

  const stop = () => {
    continue_play.value = false;
  };

  return {
    current_music,
    music_list,
    continue_play,
    set_music_list,
    set_current_music,
    is_playing,
    set_is_playing,
    play,
    stop,
  };
});
