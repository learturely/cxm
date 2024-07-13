<script lang="ts">
  import { Input } from "$lib/components/ui/input/index.js";
  import { Label } from "$lib/components/ui/label/index.js";
  import { Switch } from "$lib/components/ui/switch/index.js";
  import { emit } from "@tauri-apps/api/event";
  import M3u8Video from "./components/M3u8Video.svelte";
  import ListRooms from "./components/ListRoom.svelte";
  import {
    canUseCam,
    canUseCap,
    canUseHls,
    getQrCodeTypeCount,
    Page,
    type Room,
  } from "$lib/commands/tools";
  import type { LiveUrlPair, RoomPair } from "$lib/commands/xddcc";
  import { Button } from "$lib/components/ui/button/index.js";
  import * as RadioGroup from "$lib/components/ui/radio-group/index.js";
  import {
    checkPermissions,
    requestPermissions,
    scan,
  } from "@tauri-apps/plugin-barcode-scanner";
  export let scanning: boolean = false;
  export let state: Page = Page.sign;
  export let videoPlayer: { captureEnc: () => Promise<string> };
  export let unames = new Set<string>();
  let locationStr: string = "";
  let showUrlType: "所有设备号" | "当前设备号" = "所有设备号";
  let rooms: RoomPair[] = [];
  let userRooms: LiveUrlPair[] = [];
  let searching = false;
  let showUrl = false;
  let room: Room;
  let src = "";
  const useHls = canUseHls();
  const useCam = canUseCam();
  const useCap = canUseCap();
  const qrCodeGetterCount = getQrCodeTypeCount();
  let getQrCodeType: "live" | "scan" | "cap" = "live";
  if (useHls) {
    getQrCodeType = "live";
  } else if (useCam) {
    getQrCodeType = "scan";
  } else {
    getQrCodeType = "cap";
  }
  $: emit("sign:qrcode:location", {
    location_str: locationStr,
  }).then();
  async function onFindUrl(event: { detail: string }) {
    src = event.detail;
    if (state != Page.livePlayer) {
      state = Page.livePlayer;
      window.history.pushState(
        { state: Page.livePlayer },
        "",
        "?state=1&page=QrCode?Player"
      );
    }
  }
  async function qrCodeSign() {
    if (getQrCodeType == "live") {
      let enc = await videoPlayer.captureEnc();
      enc ? await emit("sign:qrcode:enc", enc) : {};
    } else if (getQrCodeType == "scan") {
      let enc = await scanQrCode();
      enc ? await emit("sign:qrcode:enc", enc) : {};
    } else if (getQrCodeType == "cap") {
      await emit("sign:qrcode:enc", "");
    }
  }
  async function scanQrCode(): Promise<string> {
    let perm = await checkPermissions();
    if (perm == "prompt" || perm == "denied" || perm == null) {
      perm = await requestPermissions();
    }
    if (perm == "granted") {
      scanning = true;
      state = Page.qrCodeScanner;
      window.history.pushState(
        { state: Page.qrCodeScanner },
        "",
        "?state=1&page=QRCODESCAN"
      );
      let scanned = await scan();
      if (scanned) {
        const url = new URL(scanned.content);
        let params = url.searchParams;
        window.history.back();
        return params.get("enc");
      }
    } else return null;
  }
</script>

<div class="flex-col space-y-2">
  <Input bind:value={locationStr} inputmode="text" placeholder="位置" />
  <div class="flex items-center space-x-2">
    {#if qrCodeGetterCount > 1}
      <RadioGroup.Root bind:value={getQrCodeType}>
        <div class="flex items-center space-x-2">
          {#if useHls}
            <RadioGroup.Item value="live" id="r1" />
            <Label for="r1">魔法</Label>
          {/if}
          {#if useCam}
            <RadioGroup.Item value="scan" id="r2" />
            <Label for="r2">扫码</Label>
          {/if}
          {#if useCap}
            <RadioGroup.Item value="cap" id="r3" />
            <Label for="r3">截屏</Label>
          {/if}
        </div>
        <RadioGroup.Input name="spacing" />
      </RadioGroup.Root>
    {/if}
    <Button
      disabled={qrCodeGetterCount == 0 ||
        (getQrCodeType == "live" && state != Page.livePlayer)}
      on:click={async () => {
        await qrCodeSign();
      }}
    >
      签到
    </Button>
  </div>
  {#if getQrCodeType == "live" && useHls}
    {#if state == Page.livePlayer}
      <M3u8Video bind:this={videoPlayer} {src} />
    {:else}
      <ListRooms
        bind:room
        bind:showUrl
        bind:searching
        bind:showUrlType
        bind:rooms
        bind:userRooms
        {unames}
        on:findUrl={onFindUrl}
      />
    {/if}
  {/if}
</div>
