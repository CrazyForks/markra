import { providerRequiresApiKey } from "./providers";

describe("AI provider auth", () => {
  it("marks local providers as not requiring API keys", () => {
    expect(providerRequiresApiKey({ id: "ollama", type: "ollama" })).toBe(false);
    expect(providerRequiresApiKey({ id: "openai", type: "openai" })).toBe(true);
    expect(providerRequiresApiKey({ id: "custom-provider-1", type: "openai-compatible" })).toBe(true);
  });
});
