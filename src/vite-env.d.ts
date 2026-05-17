/// <reference types="vite/client" />

interface ImportMetaEnv {
  /** 可选：覆盖默认 API 根地址，例如 http://127.0.0.1:8080 */
  readonly VITE_BASE_URL?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}

declare module '*.vue' {
  import type {DefineComponent} from 'vue';
  const component: DefineComponent<{}, {}, any>;
  export default component;
}
