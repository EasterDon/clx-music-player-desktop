import {invoke} from '@tauri-apps/api/core';

export const delay = async (delay_time:number):Promise<boolean> => {
  return new Promise((resolve)=>{
    setTimeout(()=>resolve(true),delay_time);
  });
};

export const click = async (music_char:string) => {
  const char_array = Array.from(music_char);
  await Promise.all(char_array.map((item)=>invoke('click',{key:item})));
};
