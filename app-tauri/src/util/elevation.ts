import { invoke } from "@tauri-apps/api/core";

export interface ElevationState {
  isElevated: boolean;
  launchAsAdministrator: boolean;
}

export const get_elevation_state = () =>
  invoke<ElevationState>("get_elevation_state");

export const set_launch_as_administrator = (enabled: boolean) =>
  invoke<void>("set_launch_as_administrator", { enabled });

export const restart_as_admin = () => invoke<void>("restart_as_admin");

export const take_startup_elevation_toast = () =>
  invoke<string | null>("take_startup_elevation_toast");
