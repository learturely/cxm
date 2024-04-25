import { invoke } from "@tauri-apps/api/core";
import {
  CAN_USE_CAM,
  CAN_USE_CAP,
  CAN_USE_HLS,
  GET_QR_CODE_TYPE_COUNT,
  OS_NAME,
} from "./constants";

export type Room = { code: string; url: VideoPath };
export type VideoPath = {
  ppt_video: string | null;
  teacher_full: string | null;
  teacher_track: string | null;
  student_full: string | null;
};
export enum Page {
  home,
  login,
  courseSigns,
  sign,
  livePlayer,
  qrCodeScanner,
  // locations,
  // locationImpoter,
}
// export type HomePageData = {
//   value: string;
// };
// // export type LoginPageData = {};
// export type CourseSignsPageData = {};
// export type SignPageData = {};
// export type LivePlayerPageData = {};
// export type GlobalStateData<T extends Page = Page.home> = T extends Page.home
//   ? HomePageData
//   : T extends Page.courseSigns
//   ? CourseSignsPageData
//   : T extends Page.sign
//   ? SignPageData
//   : T extends Page.livePlayer
//   ? LivePlayerPageData
//   : null;
// // export type qrCodeScannerPageData = {};
// export class GlobalState {
//   page: Page;
//   data: GlobalStateData<typeof this.page>;
// }
export async function scanImage(
  w: number,
  h: number,
  imageBuffer: Iterable<number>
): Promise<string> {
  return await invoke<string>("scan_image", {
    w,
    h,
    imageBuffer: Array.from(imageBuffer),
  });
}
export function osName(): string {
  return OS_NAME;
}
export function canUseHls(): boolean {
  return CAN_USE_HLS;
}
export function canUseCam(): boolean {
  return CAN_USE_CAM;
}
export function canUseCap(): boolean {
  return CAN_USE_CAP;
}
export function getQrCodeTypeCount(): number {
  return GET_QR_CODE_TYPE_COUNT;
}
