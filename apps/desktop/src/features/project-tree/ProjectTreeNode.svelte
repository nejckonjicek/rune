<script lang="ts">
    import type { InstanceSnapshot } from "../../lib/project/projectTypes";

    export let node: InstanceSnapshot
    export let selectedId: number | null = null
    export let onSelect: ( node: InstanceSnapshot ) => void = () => {}
</script>

<li class="tree-node">
    <button
        type="button"
        class:selected={selectedId === node.id}
        aria-current={selectedId === node.id ? "true" : undefined}
        onclick={() => onSelect(node)}
    >
        <span>{node.name}</span>
        <small>{node.className}</small>
    </button>

    {#if node.children.length > 0}
        <ul class="tree-children">
            {#each node.children as child}
                <svelte:self
                    node={child}
                    {selectedId}
                    {onSelect}
                />
            {/each}
        </ul>
    {/if}
</li>
