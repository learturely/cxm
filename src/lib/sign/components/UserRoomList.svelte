<script lang="ts">
    import * as Table from "$lib/components/ui/table";
    import {ScrollArea} from "$lib/components/ui/scroll-area/index.js";
    import {
        code2Url,
        type LiveUrlPair,
        type RoomPair,
    } from "$lib/commands/xddcc";
    import type {Room, VideoPath} from "$lib/commands/tools";

    export let searching: boolean;
    export let room: Room;
    export let userRooms: LiveUrlPair[] = [];

    // let urlLineCount = -1;
    async function onClick(
        name: string,
        room_: RoomPair,
        live: VideoPath,
        line: number
    ) {
        // urlLineCount = line;
        room = {code: room_.code, url: live};
    }
</script>

<Table.Root>
    <Table.Header>
        <Table.Row>
            <Table.Head class="w-[100px]">姓名</Table.Head>
            <Table.Head>教室</Table.Head>
            <Table.Head class="text-right">设备码</Table.Head>
        </Table.Row>
    </Table.Header>
</Table.Root>
<ScrollArea class="flex-col h-32 rounded-md" orientation="vertical">
    <Table.Root>
        <Table.Caption>教室列表</Table.Caption>
        <Table.Body>
            {#each userRooms as room, index (index)}
                <Table.Row
                        on:click={async () => {
            searching = true;
            await onClick(room.name, room.room, room.live, index);
            searching = false;
          }}
                >
                    <Table.Cell class="font-medium">{room.name}</Table.Cell>
                    <Table.Cell class="text-right">{room.room.name}</Table.Cell>
                    <Table.Cell class="text-right">{room.room.code}</Table.Cell>
                </Table.Row>
            {/each}
        </Table.Body>
    </Table.Root>
</ScrollArea>
