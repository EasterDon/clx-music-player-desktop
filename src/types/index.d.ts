type MusicListItem = {
  id: number,
  author: string,
  name: string,
  cover_url: string,
  mp3_url: string
};

type MusicList = Array<MusicListItem>;

type MusicChars = Array<string | number>;

type MusicScriptPlayerState = {
  songs_list:MusicList,
  current_song_value:null | MusicListItem,
  continue_play:boolean
};
