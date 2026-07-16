<script lang="ts">
    import type { InstanceSnapshot } from "../../lib/project/projectTypes";
    import ProjectTreeNode from "./ProjectTreeNode.svelte"

    export let root: InstanceSnapshot
    export let onSelect: (node: InstanceSnapshot) => void = () => {}
    let selectedId: number | null = null

    function selectNode(node: InstanceSnapshot) {
      selectedId = node.id
      onSelect(node)
    }
</script>

<ul class="project-tree" aria-label="Project tree">
    <ProjectTreeNode
        node={root}
        {selectedId}
        onSelect={selectNode}
    />
</ul>
