import type { Client as NativeClient } from './index'

export type ClientAuditTarget = 'stdout' | { file: string }

export interface ClientAuditOptions {
  target?: ClientAuditTarget
  redact?: string[]
}

export interface ClientWatchOptions {
  enable?: boolean
  paths?: string[]
  debounceMs?: number
}

export interface ClientOptions {
  model?: string
  providersDir?: string
  watch?: ClientWatchOptions | null
  audit?: ClientAuditOptions | null
}

export interface PromptMessage {
  role: string
  content: string
}

export interface CreatePromptOptions {
  messages: PromptMessage[]
  sampling?: Record<string, unknown>
  strictMode?: string
  response?: Record<string, unknown>
  tools?: Array<Record<string, unknown>>
  toolChoice?: string | { name: string }
  metadata?: Record<string, unknown>
}

export interface SimplePromptOptions {
  message?: string
  user?: string
  system?: string
  temperature?: number
  sampling?: Record<string, unknown>
  strictMode?: string
}

export declare class Client extends NativeClient {
  constructor(provider: string, options?: ClientOptions | null)
  complete(prompt: any): Promise<any>
  completeFile(path: string): Promise<any>
  completeText(message: string, options?: SimplePromptOptions): Promise<any>
}

export declare function loadPrompt(path: string): any
export declare function createPrompt(options: CreatePromptOptions): any
export declare function simplePrompt(options?: SimplePromptOptions): any
