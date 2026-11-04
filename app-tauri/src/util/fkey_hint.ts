import { invoke } from "@tauri-apps/api/core";

/**
 * F1/F2 提示文案。
 *
 * 平台判断在 Rust 端（`clx_core::fkey_hint`）完成，与 GPUI 版共用同一实现：
 * macOS 未启用「标准功能键」时标签为 `fn+F1` / `fn+F2`，否则为 `F1` / `F2`。
 */
export interface FkeyHint {
  /** 是否需要配合 fn 键。 */
  needsFn: boolean;
  f1Label: string;
  f2Label: string;
  startHint: string;
  stopHint: string;
}

/** 取不到时回退为不带 fn 的写法（Windows/Linux 与 macOS 已开启标准功能键的情况）。 */
const fallback: FkeyHint = {
  needsFn: false,
  f1Label: "F1",
  f2Label: "F2",
  startHint: "F1开始自动演奏",
  stopHint: "按 F2 可停止",
};

export const get_fkey_hint = async (): Promise<FkeyHint> => {
  try {
    return await invoke<FkeyHint>("get_fkey_hint");
  } catch {
    return fallback;
  }
};
