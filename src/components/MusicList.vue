<script setup lang="ts">
import { reactive, ref } from "vue";
import {
  Avatar,
  Card,
  CardMeta,
  List,
  ListItem,
  ConfigProvider,
} from "ant-design-vue";
import zhCN from "ant-design-vue/es/locale/zh_CN";
import { useBreakpoints } from "@vueuse/core";
import { songs_resource_url } from "@/config";
const { songs } = defineProps<{ songs: MusicList }>();

const breakpoints = useBreakpoints({
  line: 1440,
});

const isLine = breakpoints.smaller("line");

const cover_src = (song: MusicListItem) => {
  return `${songs_resource_url}/${song.id}/music.jpg`;
};

const emit = defineEmits(["set_current_song"]);
const set_current_song = (item: MusicListItem) => {
  if (current_song_id.value === item.id) return;
  current_song_id.value = item.id;
  emit("set_current_song", item);
};
const current_song_id = ref<number | null>(null);

const img_loaded = reactive<boolean[]>([]);
const loaded = (id: number) => {
  img_loaded[id] = true;
};
</script>

<template>
  <div class="music-list">
    <div class="mobile" v-show="isLine">
      <ConfigProvider :locale="zhCN">
        <List size="large" item-layout="horizontal" :data-source="songs">
          <template #renderItem="{ item }">
            <ListItem
              class="line-item-container"
              :class="{ 'mobile-selected': current_song_id === item.id }"
              @click="set_current_song(item)"
            >
              <div class="line-item">
                <Avatar :size="60" :src="cover_src(item)" />
                <div class="song-info">
                  <p>{{ item.name }}</p>
                  <p>{{ item.author }}</p>
                </div>
              </div>
            </ListItem>
          </template>
        </List>
      </ConfigProvider>
    </div>
    <div class="desktop" v-show="!isLine">
      <Card
        class="card"
        :class="{
          hide: !img_loaded[item.id],
          'desktop-selected': current_song_id === item.id,
        }"
        hoverable
        v-for="item in songs"
        :key="item.id"
        @click="set_current_song(item)"
      >
        <template #cover>
          <img :src="cover_src(item)" alt="" :onload="() => loaded(item.id)" />
        </template>
        <CardMeta>
          <template #title>
            <span class="title">{{ item.name }}</span></template
          >
          <template #description>{{ item.author }}</template>
        </CardMeta>
      </Card>
    </div>
  </div>
</template>

<style lang="less" scoped>
.music-list {
  width: 100%;
}

.mobile {
  width: 100%;
}

.line-item-container {
  border-radius: 5px;
  transition: all 0.5s;
  overflow: hidden;

  &:hover {
    background-color: #f0f0f0;
  }
}

.line-item {
  display: flex;
  align-items: center;
  gap: 20px;
  cursor: pointer;
  .song-info {
    flex: 1;
    p {
      text-align: left;
      margin: 0;
      padding: 0;

      &:nth-child(1) {
        font-size: 1.2rem;
        color: black;
        font-weight: 400;
      }
      &:nth-child(2) {
        font-size: 0.9rem;
        color: #696969;
      }
    }
  }
}

.desktop {
  display: grid;
  grid-template-columns: repeat(auto-fill, 150px);
  grid-gap: 20px;
  justify-content: center;
  box-sizing: border-box;
  padding: 15px 0;
}

.card {
  opacity: 1;
  transition: opacity 0.5s;
}

.hide {
  opacity: 0;
}

.title {
  overflow-wrap: break-word;
  white-space: pre-wrap;
}

.mobile-selected {
  background-color: #f0f0f0;
}

.desktop-selected {
  box-shadow: 0 0 2px 2px rgba(71, 167, 235, 0.86);
}
</style>
