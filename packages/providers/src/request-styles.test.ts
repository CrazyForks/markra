import { editableRequestStylesForProvider, requestStyleForProviderType } from "./providers";

describe("AI provider request styles", () => {
  it("exposes editable request styles only for providers that can change API style", () => {
    expect(editableRequestStylesForProvider({ id: "openai", type: "openai" })).toEqual([
      "openai-responses",
      "openai-compatible"
    ]);
    expect(editableRequestStylesForProvider({ id: "custom-provider-1", type: "openai-compatible" })).toEqual([
      "openai-responses",
      "openai-compatible",
      "anthropic",
      "google"
    ]);
    expect(editableRequestStylesForProvider({ id: "groq", type: "groq" })).toEqual([]);
  });

  it("maps provider types to their default request style", () => {
    expect(requestStyleForProviderType("openai")).toBe("openai-responses");
    expect(requestStyleForProviderType("anthropic")).toBe("anthropic");
    expect(requestStyleForProviderType("google")).toBe("google");
    expect(requestStyleForProviderType("groq")).toBe("openai-compatible");
  });
});
