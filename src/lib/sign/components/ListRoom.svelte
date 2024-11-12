<script lang="ts">
    import {Progress} from "$lib/components/ui/progress/index.js";
    import {Button} from "$lib/components/ui/button/index.js";
    import * as Table from "$lib/components/ui/table";
    import * as Accordion from "$lib/components/ui/accordion";
    import {ScrollArea} from "$lib/components/ui/scroll-area/index.js";
    import {Skeleton} from "$lib/components/ui/skeleton/index.js";
    import {Input} from "$lib/components/ui/input/index.js";
    import type {Room, VideoPath} from "$lib/commands/tools";
    import {
        code2Url,
        getLivesNow,
        listRooms,
        type LiveUrlPair,
        type RoomPair,
    } from "$lib/commands/xddcc";
    import {emit, listen} from "@tauri-apps/api/event";
    import {isStringEmpty} from "../../commands/account";
    import {createEventDispatcher} from "svelte";
    import RoomList from "./RoomList.svelte";
    import UserRoomList from "./UserRoomList.svelte";
    import ShowUrl from "./ShowUrl.svelte";

    const dispatch = createEventDispatcher();
    export let room: Room;
    export let showUrl = false;
    export let searching = false;
    let searchBtnDisabled = true;
    let searchCurrentBtnEnable = true;
    $: showUrl = !(listing || searching);
    export let uidSet = new Set<string>();
    let step = 0;
    export let rooms: RoomPair[] = [];
    export let userRooms: LiveUrlPair[] = [];
    let code = "";
    export let showUrlType: "所有设备号" | "当前设备号" = "所有设备号";

    async function onSubmit() {
        if (isURL(code)) {
            dispatch("findUrl", code);
        } else {
            searching = true;
            room = {code, url: (await code2Url(code)) as VideoPath};
            searching = false;
        }
    }

    function isURL(str: string): boolean {
        const urlRegex = /^(http|https):\/\/[^ "]+$/;
        return urlRegex.test(str);
    }

    $: searchBtnDisabled =
        ((isStringEmpty(code) || code.length != 6 || isNaN(parseInt(code))) &&
            !isURL(code)) ||
        searching ||
        !!step;
    // TODO
    $: searchCurrentBtnEnable = searchBtnDisabled;
    let step1Progress = 0;
    let step2Progress = 0;
    let listing = false;

    async function listFunction<T>(
        listClosure: () => Promise<T>,
        callback: (arg: T) => void
    ) {
        switch (step) {
            case 1:
            case 2:
                await emit("list-rooms:next-step");
                break;
            default:
                step = 1;
                listing = true;
                step1Progress = 0;
                step2Progress = 0;
                // let unlistenListLiveIdStartedPromise = listen<number>(
                //   "step1:started",
                //   (p) => {
                //     step = 1;
                //   }
                // );
                let unlistenStep2StartedPromise = listen<number>(
                    "step2:started",
                    (p) => {
                        step = 2;
                    }
                );
                let unlistenStep1Promise = listen<number>("step1:set-progress", (p) => {
                    let payload = p.payload;
                    if (step1Progress < payload) {
                        step1Progress = payload;
                    }
                    if (step1Progress == 100) {
                        step = 2;
                    }
                });
                let unlistenStep2Promise = listen<number>("step2:set-progress", (p) => {
                    let payload = p.payload;
                    if (step2Progress < payload) {
                        step2Progress = payload;
                    }
                });
                let unlistenStep2StoppedPromise = listen<number>(
                    "step2:stopped",
                    (p) => {
                        step = 0;
                    }
                );
                let roomsPromise = listClosure();
                const [
                    // unlistenListLiveIdStarted,
                    unlistenStep1,
                    unlistenStep2Started,
                    unlistenStep2,
                    unlistenStep2Stopped,
                    roomPairs,
                ] = await Promise.all([
                    // unlistenListLiveIdStartedPromise,
                    unlistenStep1Promise,
                    unlistenStep2StartedPromise,
                    unlistenStep2Promise,
                    unlistenStep2StoppedPromise,
                    roomsPromise,
                ]);
                callback(roomPairs);
                unlistenStep2Stopped();
                unlistenStep2();
                unlistenStep2Started();
                unlistenStep1();
                // unlistenListLiveIdStarted();
                listing = false;
                step = 0;
                break;
        }
    }

    async function onListRoomsBtnClick() {
        showUrlType = "所有设备号";
        await listFunction(
            async () => {
                return await listRooms(uidSet);
            },
            (roomPairs) => {
                rooms = roomPairs;
            }
        );
    }

    async function onSearchCurrentRoomsBtnClick() {
        showUrlType = "当前设备号";
        await listFunction(
            async () => {
                return await getLivesNow(uidSet);
            },
            (userRoomPairs) => {
                userRooms = userRoomPairs;
            }
        );
    }
</script>

<form
        class="flex-col justify-center w-full max-w-sm items-center space-y-2 mt-2"
        on:submit={onSubmit}
>
    <Input
            class=" mb-1 flex justify-center"
            id="code-input"
            name="code"
            type="search"
            inputmode="numeric"
            placeholder="设备码"
            bind:value={code}
    />
    <div class="flex justify-center">
        <div class="flex grow space-x-2">
            <Button disabled={searchBtnDisabled} type="submit" class="grow w-1/3">
                获取上述
            </Button>
            <Button
                    disabled={uidSet.size == 0 ||
          searching ||
          (listing && showUrlType == "当前设备号")}
                    type="button"
                    on:click={onListRoomsBtnClick}
                    class="grow  w-1/3"
            >{showUrlType == "当前设备号"
                ? "获取所有"
                : step == 1
                    ? "下一步"
                    : step == 2
                        ? "停止"
                        : "获取所有"}
            </Button>
            <Button
                    disabled={uidSet.size == 0 ||
          searching ||
          (listing && showUrlType == "所有设备号")}
                    type="button"
                    on:click={onSearchCurrentRoomsBtnClick}
                    class="grow  w-1/3"
            >{showUrlType == "所有设备号"
                ? "获取当前"
                : step == 1
                    ? "下一步"
                    : step == 2
                        ? "停止"
                        : "获取当前"}
            </Button>
        </div>
    </div>
</form>
<div class="flex-col"></div>
{#if listing}
    <div class="flex-col space-y-2 my-2">
        <div>
            <p>获取直播号</p>
            <Progress value={step1Progress}/>
        </div>
        <div>
            <p>获取设备码</p>
            <Progress value={step2Progress}/>
        </div>
    </div>
{/if}
<Accordion.Root class="w-full sm:max-w-[70%]">
    <ShowUrl {room} {searching} on:findUrl></ShowUrl>
    <Accordion.Item value="item-1">
        <Accordion.Trigger>{showUrlType}</Accordion.Trigger>
        <Accordion.Content>
            {#if showUrlType == "所有设备号"}
                {#if !listing && rooms && rooms.length}
                    <RoomList bind:room {rooms} bind:searching></RoomList>
                {/if}
            {:else if !listing && userRooms && userRooms.length}
                <UserRoomList bind:room {userRooms} bind:searching></UserRoomList>
            {/if}
        </Accordion.Content>
    </Accordion.Item>
</Accordion.Root>
