<script setup lang="ts">
import { Button, Slider } from "ant-design-vue";
import { computed } from "vue";
import {
  CaretRightOutlined,
  LeftOutlined,
  RightOutlined,
  PauseOutlined,
} from "@ant-design/icons-vue";

const emit = defineEmits<{
  audio_play: [];
  audio_pause: [];
  previous_track: [];
  next_track: [];
  seek_progress: [value: number];
  toggle_lyrics_view: [];
}>();

const props = withDefaults(
  defineProps<{
    is_playing: boolean;
    play_disabled?: boolean;
    current_time?: number;
    duration?: number;
    show_lyrics_toggle?: boolean;
  }>(),
  {
    play_disabled: true,
    current_time: 0,
    duration: 0,
    show_lyrics_toggle: false,
  },
);

const progress_max = computed(() =>
  props.duration && props.duration > 0 ? props.duration : 0,
);

const format_time = (value?: number) => {
  if (!value || Number.isNaN(value)) return "00:00";
  const seconds = Math.max(0, Math.floor(value));
  const mm = String(Math.floor(seconds / 60)).padStart(2, "0");
  const ss = String(seconds % 60).padStart(2, "0");
  return `${mm}:${ss}`;
};
</script>

<template>
  <div class="player-controls">
    <div class="progress-container">
      <span class="progress-time">{{ format_time(current_time) }}</span>
      <Slider
        class="progress-slider"
        :min="0"
        :max="progress_max"
        :value="current_time"
        :step="0.1"
        :disabled="play_disabled || progress_max <= 0"
        @change="(value) => emit('seek_progress', Number(value))"
      />
      <span class="progress-time">{{ format_time(duration) }}</span>
    </div>

    <div class="player-button-container">
      <Button shape="circle" @click="emit('previous_track')">
        <template #icon>
          <LeftOutlined />
        </template>
      </Button>

      <Button
        v-show="!is_playing"
        :disabled="play_disabled"
        shape="circle"
        size="large"
        @click="emit('audio_play')"
      >
        <template #icon>
          <CaretRightOutlined />
        </template>
      </Button>
      <Button
        v-show="is_playing"
        shape="circle"
        size="large"
        @click="emit('audio_pause')"
      >
        <template #icon>
          <PauseOutlined />
        </template>
      </Button>

      <Button shape="circle" @click="emit('next_track')">
        <template #icon>
          <RightOutlined />
        </template>
      </Button>

      <Button
        v-if="show_lyrics_toggle"
        class="lyrics-toggle-btn"
        size="small"
        @click="emit('toggle_lyrics_view')"
      >
        词
      </Button>
    </div>
  </div>
</template>

<style scoped lang="less">
.player-controls {
  width: 100%;
  border-radius: 10px;
  background-color: #efefef;
  display: flex;
  flex-direction: column;
  padding: 8px 10px;
  box-sizing: border-box;
  gap: 6px;
  position: relative;
}

.progress-container {
  display: flex;
  align-items: center;
  gap: 8px;
}

.progress-slider {
  flex: 1;
}

.progress-time {
  width: 40px;
  font-size: 12px;
  color: #666;
  text-align: center;
}

.player-button-container {
  width: 100%;
  height: 52px;
  display: flex;
  justify-content: space-evenly;
  align-items: center;

  .button {
    font-size: 2.5rem;
    color: gray;
  }
}

.lyrics-toggle-btn {
  position: absolute;
  right: 8px;
  top: 50%;
  transform: translateY(-50%);
  padding: 0 8px;
}
</style>
