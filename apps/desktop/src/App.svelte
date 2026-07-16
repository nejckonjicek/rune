<script lang="ts">
    import { open } from "@tauri-apps/plugin-dialog"
    import ProjectTree from "./features/project-tree/ProjectTree.svelte"
    import {
      getProjectErrorMessage,
      openProject,
    } from "./lib/project/projectApi"
    import type {
      InstanceSnapshot,
      ProjectSnapshot,
    } from "./lib/project/projectTypes"

    let project: ProjectSnapshot | null = null
    let selectedNode: InstanceSnapshot | null = null
    let errorMessage: string | null = null
    let isLoading = false

    async function chooseProject() {
      errorMessage = null
      isLoading = true

      try {
        const selection = await open({
          multiple: false,
          directory: false,
          filters: [
            {
              name: "Rojo project",
              extensions: ["json"],
            },
          ],
        })

        if (!selection || Array.isArray(selection)) {
          return
        }

        if (!selection.toLowerCase().endsWith(".project.json")) {
          errorMessage = "Select a Rojo .project.json file."
          return
        }

        project = await openProject(selection)
        selectedNode = null
      } catch (error) {
        errorMessage = getProjectErrorMessage(error)
      } finally {
        isLoading = false
      }
    }

    function handleNodeSelect(node: InstanceSnapshot) {
      selectedNode = node
    }
</script>

<svelte:head>
    <title>Rune</title>
</svelte:head>

<main class="app-shell">
    <header class="toolbar">
        <h1>Rune</h1>

        <button type="button" onclick={chooseProject} disabled={isLoading}>
            {isLoading ? "Loading..." : "Open project"}
        </button>
    </header>

    {#if errorMessage}
        <p class="error-message" role="alert">{errorMessage}</p>
    {/if}

    {#if project}
        <div class="workspace">
            <aside class="project-panel">
                <h2>{project.name}</h2>

                <ProjectTree
                    root={project.root}
                    onSelect={handleNodeSelect}
                />
            </aside>

            <section class="selection-panel">
                {#if selectedNode}
                    <h2>{selectedNode.name}</h2>
                    <p>{selectedNode.className}</p>
                {:else}
                    <p>Select a project node.</p>
                {/if}
            </section>
        </div>
    {:else if !isLoading}
        <section class="empty-state">
            <p>Open a Rojo project to inspect its DataModel.</p>
        </section>
    {/if}
</main>
