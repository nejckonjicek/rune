export interface ProjectSnapshot {
  name: string
  projectFile: string
  root: InstanceSnapshot
  scripts: ScriptSnapshot[]
}

export interface InstanceSnapshot {
  id: number
  name: string
  className: string
  children: InstanceSnapshot[]
}

export interface ScriptSnapshot {
  id: number
  name: string
  className: string
  relativePath: string
}

export interface ProjectCommandError {
  code: string
  message: string
}
