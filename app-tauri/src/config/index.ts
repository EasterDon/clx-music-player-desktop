/** 后端根地址：构建时通过项目根目录 .env 的 CLX_BASE_URL 注入，源码不写死线上地址 */
export const base_url = import.meta.env.CLX_BASE_URL?.trim() ?? "";
export const music_url = `${base_url}/v3/music`;
export const app_url = `${base_url}/v3/app`;
export const music_resource_url = `${base_url}/music`;

/** 统一桌面/移动断点，组件逻辑优先引用这里 */
export const desktop_breakpoint = 1020;
/** 相关位置：
 * - 逻辑层常量：src/App.vue、src/components/Lyrics/index.vue
 * - 样式层变量：src/assets/breakpoints.less
 * - 使用样式变量的文件：src/App.vue、src/components/Lyrics/index.vue、src/components/MusicList/index.vue、src/main.less
 */

export const app_version = '1.0.2';
