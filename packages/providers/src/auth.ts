import type { AiProviderConfig } from "./types";

export function providerRequiresApiKey(provider: Pick<AiProviderConfig, "id" | "type">) {
  return provider.type !== "ollama";
}
