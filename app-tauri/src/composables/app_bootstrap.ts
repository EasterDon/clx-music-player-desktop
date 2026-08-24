import { onMounted } from "vue";
import { message } from "ant-design-vue";
import { take_startup_elevation_toast } from "@/util/elevation";

/**
 * 应用启动：拉曲库与软件信息（全局快捷键按演奏模式另行注册）。
 */
export const use_app_bootstrap = (
  load_music_list: () => Promise<void>,
  load_app_info: () => Promise<void>,
) => {
  onMounted(async () => {
    try {
      const tip = await take_startup_elevation_toast();
      if (tip) message.warn(tip, 6);
    } catch {
      /* 非 Tauri 环境忽略 */
    }

    try {
      await load_music_list();
    } catch (err) {
      message.error(
        err instanceof Error ? err.message : "获取歌曲列表失败",
      );
    }

    try {
      await load_app_info();
    } catch (err) {
      message.error(
        err instanceof Error ? err.message : "获取软件信息失败",
      );
    }
  });
};
