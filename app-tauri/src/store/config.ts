import { ref } from "vue";
import { defineStore } from "pinia";
import { app_version } from "@/config";

/** 与设置抽屉 Radio 的 value 一致，使用字符串避免 Ant Design 将数字枚举变成字符串导致比较失败。 */
export enum PlayMode {
  listen = "listen",
  script = "script",
}

export const is_script_play_mode = (mode: PlayMode | string) =>
  mode === PlayMode.script || mode === "script";

export const use_config_store = defineStore("config", () => {
  const play_mode = ref<PlayMode>(PlayMode.listen);

  const app_info = ref<App_Info>({
    id: 1,
    announcement: "",
    app_version,
    app_download_link: "",
    app_version_description: "",
  });

  return {
    play_mode,
    app_info,
  };
});
