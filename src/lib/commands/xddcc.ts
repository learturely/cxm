import { invoke } from "@tauri-apps/api/core";
import type { VideoPath } from "./tools";
export type RoomPair = {
  name: string;
  code: string;
};
export type LiveUrlPair = {
  name: string;
  room: RoomPair;
  live: VideoPath;
};
export async function listRooms(unames_: Set<string>): Promise<RoomPair[]> {
  let unames = Array.from(unames_);
  return await invoke("list_rooms", { unames });
}
export async function code2Url(code: string): Promise<VideoPath> {
  return await invoke<VideoPath>("code_to_video_path", {
    code,
  });
}
export async function getLivesNow(
  unames_: Set<string>
): Promise<LiveUrlPair[]> {
  let unames = Array.from(unames_);
  return await invoke<LiveUrlPair[]>("get_video_pathes_now", {
    unames,
  });
}
