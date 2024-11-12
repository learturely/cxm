import {invoke} from "@tauri-apps/api/core";
import type {Course} from "./course";
import type {AccountPair} from "./account";

export type RawSign = {
    name: string;
    start_timestamp: number;
    active_id: string;
    course: Course;
    other_id: string;
    status_code: number;
};
export type RawSignPair = {
    sign: RawSign;
    account_pairs: AccountPair[];
};

export enum SignType {
    photo = "photo",
    normal = "normal",
    qrcode = "qrcode",
    gesture = "gesture",
    location = "location",
    signcode = "signcode",
    unknown = "unknown",
}

export async function listCourseActivities(course: Course): Promise<RawSign[]> {
    let activities = [];
    await invoke<RawSign[]>("list_course_activities", {course})
        .then((data) => {
            activities = data;
        })
        .catch((error) => {
            console.error(error);
        });
    return activities;
}

export async function listAllActivities(): Promise<RawSignPair[]> {
    let activities = [];
    await invoke<RawSignPair[]>("list_all_activities")
        .then((data) => {
            activities = data;
        })
        .catch((error) => {
            console.error(error);
        });
    return activities;
}

export async function prepareSign(sign: RawSign, accounts: AccountPair[]) {
    await invoke<void>("prepare_sign", {sign, accounts});
}

export async function getSignType(): Promise<SignType> {
    return (await invoke<string>("get_sign_type")) as SignType;
}

export async function signSingle() {
    await invoke<void>("sign_single").catch((error) => {
        console.error(error);
    });
}

export async function removeUid(uid: string): Promise<boolean> {
    return await invoke<boolean>("remove_uid", {uid});
}

export async function addUid(uid: string): Promise<boolean> {
    return await invoke<boolean>("add_uid", {uid});
}

export async function extentUidSet(uidSet: Set<string>): Promise<void> {
    let uidVec = Array.from(uidSet);
    return await invoke<void>("extent_uid_set", {uidVec});
}

export async function hasUid(uid: string): Promise<boolean> {
    return await invoke<boolean>("has_uid", {uid});
}

export async function clearUidSet(): Promise<void> {
    return await invoke<void>("clear_uid_set");
}
