<script lang="ts">
  import * as Table from "$lib/components/ui/table";
  import * as Accordion from "$lib/components/ui/accordion";
  import { Skeleton } from "$lib/components/ui/skeleton/index.js";
  import type { Room } from "$lib/commands/tools";
  import { createEventDispatcher } from "svelte";
  export let room: Room;
  export let searching: boolean;
  const dispatch = createEventDispatcher();
</script>

<Accordion.Item value="item-2">
  {#if searching}
    <Skeleton class="h-8 m-3" />
  {:else}
    <Accordion.Trigger>
      {#if room}
        {room.code}
      {:else}
        直播地址
      {/if}
    </Accordion.Trigger>
  {/if}
  <Accordion.Content>
    {#if room}
      <Table.Root>
        <Table.Body>
          {#if room.url.ppt_video}
            <Table.Row
              class="font-medium"
              on:click={() => {
                dispatch("findUrl", room.url.ppt_video);
              }}
            >
              pptVideo
            </Table.Row>
          {/if}
          {#if room.url.teacher_full}
            <Table.Row
              class="font-medium"
              on:click={() => {
                dispatch("findUrl", room.url.teacher_full);
              }}
            >
              teacherFull
            </Table.Row>
          {/if}
          {#if room.url.teacher_track}
            <Table.Row
              class="font-medium"
              on:click={() => {
                dispatch("findUrl", room.url.teacher_track);
              }}
            >
              teacherTrack
            </Table.Row>
          {/if}
          {#if room.url.student_full}
            <Table.Row
              class="font-medium"
              on:click={() => {
                dispatch("findUrl", room.url.student_full);
              }}
            >
              studentFull
            </Table.Row>
          {/if}
        </Table.Body>
      </Table.Root>
    {/if}
  </Accordion.Content>
</Accordion.Item>
