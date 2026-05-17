import type { SpinProps } from 'ant-design-vue';
import { createSpinComponent } from './spin';

export type SpinConfig = SpinProps & {
  target?: HTMLElement | string;
};

export type SpinInstance = ReturnType<typeof createSpinComponent>;
