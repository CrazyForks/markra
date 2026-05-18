import {
  aiProviderRequestStyles,
  type AiProviderApiStyle,
  type AiProviderConfig,
  type AiProviderRequestStyle
} from "./types";

const openAiEditableRequestStyles: AiProviderRequestStyle[] = ["openai-responses", "openai-compatible"];

export function isAiProviderRequestStyle(value: unknown): value is AiProviderRequestStyle {
  return typeof value === "string" && aiProviderRequestStyles.includes(value as AiProviderRequestStyle);
}

export function requestStyleForProviderType(type: AiProviderApiStyle): AiProviderRequestStyle {
  if (type === "openai") return "openai-responses";
  if (type === "anthropic") return "anthropic";
  if (type === "google") return "google";

  return "openai-compatible";
}

export function editableRequestStylesForProvider(
  provider: Pick<AiProviderConfig, "id" | "type">
): AiProviderRequestStyle[] {
  if (isCustomAiProviderId(provider.id)) return [...aiProviderRequestStyles];
  if (provider.type === "openai") return [...openAiEditableRequestStyles];

  return [];
}

function isCustomAiProviderId(providerId: string) {
  return providerId.startsWith("custom-provider-");
}
