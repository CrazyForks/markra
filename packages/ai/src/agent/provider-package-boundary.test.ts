import { createDefaultAiSettings } from "@markra/providers";

import { getChatAdapterForProvider } from "./chat-adapters";

describe("provider package boundary", () => {
  it("builds chat requests from provider settings owned by @markra/providers", () => {
    const provider = createDefaultAiSettings().providers.find((candidate) => candidate.id === "openai");
    if (!provider) throw new Error("OpenAI provider template is missing.");
    if (!provider.defaultModelId) throw new Error("OpenAI provider default model is missing.");

    const request = getChatAdapterForProvider(provider).buildRequest(provider, provider.defaultModelId, [
      { content: "Say ok.", role: "user" }
    ]);

    expect(request.url).toBe("https://api.openai.com/v1/responses");
    expect(request.body).toMatchObject({
      input: [{ content: [{ text: "Say ok.", type: "input_text" }], role: "user", type: "message" }],
      model: provider.defaultModelId
    });
  });
});
