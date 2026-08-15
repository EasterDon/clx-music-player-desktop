import { message } from "ant-design-vue";
import { register, unregisterAll } from "@tauri-apps/plugin-global-shortcut";
import { get_music_notation } from "@/api";
import { is_script_play_mode, type PlayMode } from "@/store/config";
import { use_music_store } from "@/store/player";

/**
 * 演奏模式：F1 拉谱演奏，F2 停止。聆听模式不注册全局快捷键。
 */
export const use_global_shortcuts = () => {
  const music_store = use_music_store();

  const register_script_shortcuts = async () => {
    await unregisterAll();

    await register(["F1", "F2"], async (event) => {
      if (event.state !== "Pressed") return void 0;

      const key = event.shortcut.replace(/\s+/g, "").toUpperCase();

      if (key === "F1") {
        if (!music_store.current_music) {
          message.warn("请先选择一首歌曲哦");
          return void 0;
        }
        try {
          const music_notation = await get_music_notation(
            music_store.current_music.id,
          );
          await music_store.play(music_notation);
        } catch (err) {
          message.error(
            err instanceof Error ? err.message : "获取谱面失败",
          );
        }
        return void 0;
      }

      if (key === "F2") {
        if (!music_store.current_music) return void 0;
        music_store.stop();
      }
    });
  };

  const sync_shortcuts_with_play_mode = async (mode: PlayMode) => {
    try {
      if (is_script_play_mode(mode)) {
        await register_script_shortcuts();
      } else {
        await unregisterAll();
      }
    } catch (err) {
      message.error(
        err instanceof Error
          ? `快捷键注册失败：${err.message}`
          : "快捷键注册失败",
      );
    }
  };

  return { sync_shortcuts_with_play_mode };
};
