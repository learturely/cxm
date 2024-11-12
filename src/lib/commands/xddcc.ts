import {invoke} from "@tauri-apps/api/core";
import type {VideoPath} from "./tools";

export type RoomPair = {
    name: string;
    code: string;
};
export type LiveUrlPair = {
    name: string;
    room: RoomPair;
    live: VideoPath;
};

export async function listRooms(uidSet: Set<string>): Promise<RoomPair[]> {
    let uidVec = Array.from(uidSet);
    return await invoke("list_rooms", {uidVec});
}

export async function code2Url(code: string): Promise<VideoPath> {
    return await invoke<VideoPath>("code_to_video_path", {
        code,
    });
}

export async function getLivesNow(
    uidSet: Set<string>
): Promise<LiveUrlPair[]> {
    let uidVec = Array.from(uidSet);
    return await invoke<LiveUrlPair[]>("get_video_paths_now", {
        uidVec,
    });
}
