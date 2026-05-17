/** 默认线上地址；本地可在 .env 中设置 VITE_BASE_URL */
export const base_url = import.meta.env.VITE_BASE_URL ?? 'http://127.0.0.1';
export const music_url = `${base_url}/v3/music`;
export const app_url = `${base_url}/v3/app`;
export const music_resource_url = `${base_url}/music`;

/** 统一桌面/移动断点，组件逻辑优先引用这里 */
export const desktop_breakpoint = 1200;

export const app_version = '1.0.1';
