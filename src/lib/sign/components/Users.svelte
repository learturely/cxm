<script lang="ts">
    import {ScrollArea} from "$lib/components/ui/scroll-area/index.js";
    import * as Avatar from "$lib/components/ui/avatar";
    import * as Table from "$lib/components/ui/table/index.js";
    import {Checkbox} from "$lib/components/ui/checkbox";
    import {Label} from "$lib/components/ui/label";
    import {type AccountPair} from "$lib/commands/account";
    import {extentUidSet, clearUidSet} from "$lib/commands/sign";
    import {listen, type UnlistenFn} from "@tauri-apps/api/event";
    import {onDestroy, onMount} from "svelte";

    export let uidSet: Set<string> = new Set([]);
    export let accounts: AccountPair[] = [];
    export let disabled = false;
    let unlistenSusses: UnlistenFn, unlistenFail: UnlistenFn;
    let resultsFailedCount = 0;
    for (const account of accounts) {
        uidSet.add(account.uid);
    }
    $: if (uidSet.size == resultsFailedCount) {
        resultsFailedCount = 0;
        updateUidSet().then();
        disabled = false;
    }
    uidSet = uidSet;
    extentUidSet(uidSet).then();

    async function listenSignResults() {
        let success = listen<string>("sign:susses", (e) => {
            let uid = e.payload;
            let p = document.getElementById(
                "sign-result-msg-" + uid
            ) as HTMLParagraphElement;
            p.textContent = "签到成功";
            p.className = "text-right truncate text-green-600";
            removeOrAddElement(uid);
            setTimeout(() => {
                p.textContent = "";
                p.className = "text-right truncate";
            }, 2500);
        });
        let fail = listen<string[]>("sign:fail", (e) => {
            let [uid, msg] = e.payload;
            let p = document.getElementById(
                "sign-result-msg-" + uid
            ) as HTMLParagraphElement;
            p.textContent = msg;
            p.className = "text-right truncate text-red-600";
            resultsFailedCount += 1;
            setTimeout(() => {
                p.textContent = "";
                p.className = "text-right truncate";
            }, 2500);
        });
        [unlistenSusses, unlistenFail] = await Promise.all([success, fail]);
    }

    function removeOrAddElement(uid: string) {
        if (uidSet.has(uid)) {
            uidSet.delete(uid);
        } else {
            uidSet.add(uid);
        }
        uidSet = uidSet;
    }

    async function updateUidSet() {
        await clearUidSet();
        await extentUidSet(uidSet);
        console.log("update uidSet");
        console.log(uidSet);
    }

    onMount(listenSignResults);
    onDestroy(() => {
        unlistenFail();
        unlistenSusses();
    });
</script>

<div class="items-center justify-center">
    <ScrollArea class="h-48 rounded-md border">
        <Table.Root>
            <Table.Body>
                {#each accounts as account (account)}
                    <div class="flex flex-row items-center space-x-2 ml-4">
                        <Checkbox
                                {disabled}
                                checked={uidSet.has(account.uid)}
                                id={"ulcb-" + account.uid}
                                onCheckedChange={async () => {
                removeOrAddElement(account.uid);
                await updateUidSet();
              }}
                        />
                        <Label class="flex grow" for={"ulcb-" + account.uid}>
                            <Table.Row>
                                <Table.Cell>
                                    <div class="flex flex-row items-center space-x-2 grow">
                                        <Avatar.Root class="size-7">
                                            <Avatar.Image src={account.avatar} alt={account.name}/>
                                            <Avatar.Fallback>{account.name.at(0)}</Avatar.Fallback>
                                        </Avatar.Root>
                                        <p>
                                            {account.name}
                                        </p>
                                        <div class="flex flex-row-reverse grow">
                                            <p
                                                    class="text-right truncate"
                                                    id={"sign-result-msg-" + account.uid}
                                            ></p>
                                        </div>
                                    </div>
                                </Table.Cell>
                            </Table.Row>
                        </Label>
                    </div>
                {/each}
            </Table.Body>
        </Table.Root>
    </ScrollArea>
</div>
