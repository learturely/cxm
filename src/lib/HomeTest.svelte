<script lang="ts">
  import { type Room } from "./commands/tools";
  import testQrCode from "$lib/tests/1.png";
  // import html2canvas from "html2canvas";
  import M3u8Video from "./sign/components/M3u8Video.svelte";
  import ListRooms from "./sign/components/ListRoom.svelte";
  import { isStringEmpty } from "./commands/account";
  let enc = "none";
  let err = "none";
  export let unames = new Set<string>();
  let searching = false;
  let showUrl = false;
  let room: Room;
  let src = "";
  let videoPlayer: { captureEnc: () => Promise<string> };
  async function onClick() {
    // let imgElement = window.document.getElementById(
    //   "test-img"
    // ) as HTMLImageElement;
    // html2canvas(imgElement, {
    //   scale: 0.75,
    // })
    //   .then((canvas) => {
    //     const dw = canvas.width;
    //     const dh = canvas.height;
    //     let data = canvas.getContext("2d").getImageData(0, 0, dw, dh).data;
    //     console.log(data);
    //     // let buffer = Array.from(data);
    //     // console.log(buffer);
    //     scanImage(dw, dh, data)
    //       .then((enc_) => {
    //         enc = enc_;
    //       })
    //       .catch((err_) => {
    //         err = err_;
    //       });
    //   })
    //   .catch((err_) => {
    //     err = err_;
    //   });
    await videoPlayer
      .captureEnc()
      .then((enc_) => {
        enc = enc_;
      })
      .catch((err_) => {
        err = err_;
      });
  }
  async function onFindUrl(event: { detail: string }) {
    src = event.detail;
  }
</script>

<p>{enc}</p>
<p>{err}</p>
<p>{src}</p>
<!-- svelte-ignore a11y-click-events-have-key-events -->
<!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
<img
  id="test-img"
  src={testQrCode}
  alt="test"
  on:click={onClick}
  on:load={() => {
    console.log("loaded");
  }}
/>
<ListRooms
  bind:room
  bind:showUrl
  bind:searching
  {unames}
  on:findUrl={onFindUrl}
/>
{#if !isStringEmpty(src)}
  <M3u8Video bind:this={videoPlayer} {src} />
{/if}
