<script setup lang="ts">
import {computed, onMounted,useTemplateRef} from 'vue';
import {MusicScriptPlayer} from '@/class/music_script_player';
import {songs_resource_url} from '@/config';

const music_player = new MusicScriptPlayer();
const current_music_mp3_url = computed(() => {
  if(music_player.current_song_value?.mp3_url){
    return songs_resource_url + music_player.current_song_value?.mp3_url;
  }else{
    return '';
  }
});
const set_current_song = (id:number) => {
  music_player.set_current_music(id);
};
onMounted(async()=>{
  await music_player.set_hotkey();
  await music_player.init_songs_list();
});
const audio = useTemplateRef('audio');
const audio_play = () => {
  audio.value?.play();
};

const audio_pause = () => {
  audio.value?.pause();
};
</script>

<template>
<button @click="audio_play">播放</button>
<button @click="audio_pause">暂停</button>
<p>按下F1开始播放脚本，F2停止播放脚本，请一定要在游戏弹琴界面再按</p>
<p v-for="item in music_player.songs_list" :key="item.id"
@click="set_current_song(item.id)" class="song"
>
    {{item.name}}
</p>
<p>{{music_player.current_song_value
  ?`当前选择：${music_player.current_song_value.name}`:"还没选择音乐哦"}}</p>
<audio ref="audio" class="audio" :src="current_music_mp3_url" controls/>
</template>

<style lang="less" scoped>
.audio{
  display: none;
}

.song{
color: gray;
}
</style>
