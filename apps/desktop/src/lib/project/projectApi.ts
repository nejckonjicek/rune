import { invoke } from "@tauri-apps/api/core"
import type { ProjectSnapshot } from "./projectTypes"

export function openProject(path: string): Promise<ProjectSnapshot> {
  return invoke<ProjectSnapshot>("open_project", { path })
}

export function getProjectErrorMessage(error: unknown): string {
  if (typeof error === "string") {
    return error
  }

  if (typeof error === "object" && error !== null) {
    const candidate = error as Partial<{ message: unknown }>

    if (typeof candidate.message === "string") {
      return candidate.message
    }
  }

  return "Failed to load project."
}
