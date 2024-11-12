<script lang="ts">
    import * as Table from "$lib/components/ui/table";
    import {ScrollArea} from "$lib/components/ui/scroll-area/index.js";
    import {code2Url, type RoomPair} from "$lib/commands/xddcc";
    import type {Room} from "$lib/commands/tools";

    export let searching: boolean;
    export let room: Room;
    export let rooms: RoomPair[] = [];

    // let urlLineCount = -1;
    async function onClick(code: string, line: number) {
        // urlLineCount = line;
        room = {code, url: await code2Url(code)};
    }
</script>

<Table.Root>
    <Table.Header>
        <Table.Row>
            <Table.Head class="w-[100px]">教室</Table.Head>
            <Table.Head class="text-right">设备码</Table.Head>
        </Table.Row>
    </Table.Header>
</Table.Root>
<ScrollArea class="flex-col h-32 rounded-md" orientation="vertical">
    <Table.Root>
        <Table.Caption>教室列表</Table.Caption>
        <Table.Body>
            {#each rooms as room, index (index)}
                <Table.Row
                        on:click={async () => {
            searching = true;
            await onClick(room.code, index);
            searching = false;
          }}
                >
                    <Table.Cell class="font-medium">{room.name}</Table.Cell>
                    <Table.Cell class="text-right">{room.code}</Table.Cell>
                </Table.Row>
            {/each}
        </Table.Body>
    </Table.Root>
</ScrollArea>
