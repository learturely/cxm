<script lang="ts">
    import {scanImage} from "$lib/commands/tools";
    import html2canvas from "html2canvas";
    import "vidstack/bundle";

    export let src: string;
    // const hlsConfig: any = {
    //   autoStartLoad: true,
    //   enableWorker: true,
    //   maxFragLookUpTolerance: 0,
    //   maxBufferLength: 0,
    // };
    export async function captureEnc(): Promise<string> {
        let video = document.getElementsByTagName("video")[0];
        let canvas = await html2canvas(video, {scale: 2.5, useCORS: true});
        const dw = canvas.width;
        const dh = canvas.height;
        let data = canvas.getContext("2d").getImageData(0, 0, dw, dh).data;
        // console.log(data);
        return await scanImage(dw, dh, data);
    }
</script>

<media-player title="Sprite Fight" {src} autoPlay playsInline>
    <media-provider></media-provider>
    <media-video-layout
            thumbnails="https://files.vidstack.io/sprite-fight/thumbnails.vtt"
    ></media-video-layout>
</media-player>
<!-- <canvas id="test-canvas"></canvas> -->
