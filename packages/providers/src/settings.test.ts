import { normalizeAiSettings } from "./settings";

describe("AI provider settings", () => {
  it("normalizes legacy provider types into the new request API styles", () => {
    const settings = normalizeAiSettings({
      defaultProviderId: "openai",
      providers: [
        {
          enabled: true,
          id: "openai",
          models: [{ enabled: true, id: "gpt-5.5", name: "GPT-5.5" }],
          name: "OpenAI",
          type: "openai"
        },
        {
          enabled: true,
          id: "openrouter",
          models: [{ enabled: true, id: "openrouter/auto", name: "Auto" }],
          name: "OpenRouter",
          type: "openrouter"
        },
        {
          apiStyle: "amazon-bedrock",
          enabled: false,
          id: "custom-provider-1",
          models: [{ enabled: true, id: "anthropic.claude-sonnet-4-5-v1:0", name: "Claude Sonnet" }],
          name: "Legacy Bedrock Relay",
          type: "openai-compatible"
        }
      ]
    });

    expect(settings.providers.find((provider) => provider.id === "openai")).toMatchObject({
      apiStyle: "openai-responses",
      type: "openai"
    });
    expect(settings.providers.find((provider) => provider.id === "openrouter")).toMatchObject({
      apiStyle: "openai-compatible",
      type: "openrouter"
    });
    expect(settings.providers.find((provider) => provider.id === "custom-provider-1")).toMatchObject({
      apiStyle: "openai-compatible",
      type: "openai-compatible"
    });
  });

  it("keeps fixed built-in providers on their catalog request style", () => {
    const settings = normalizeAiSettings({
      defaultProviderId: "groq",
      providers: [
        {
          apiStyle: "anthropic",
          enabled: true,
          id: "groq",
          models: [{ enabled: true, id: "llama-3.3-70b-versatile", name: "Llama" }],
          name: "Groq",
          type: "groq"
        },
        {
          apiStyle: "google",
          enabled: true,
          id: "openai",
          models: [{ enabled: true, id: "gpt-5.5", name: "GPT-5.5" }],
          name: "OpenAI",
          type: "openai"
        },
        {
          apiStyle: "amazon-bedrock",
          enabled: true,
          id: "custom-provider-1",
          models: [{ enabled: true, id: "default", name: "Default" }],
          name: "Custom Provider",
          type: "openai-compatible"
        }
      ]
    });

    expect(settings.providers.find((provider) => provider.id === "groq")).toMatchObject({
      apiStyle: "openai-compatible"
    });
    expect(settings.providers.find((provider) => provider.id === "openai")).toMatchObject({
      apiStyle: "openai-responses"
    });
    expect(settings.providers.find((provider) => provider.id === "custom-provider-1")).toMatchObject({
      apiStyle: "openai-compatible"
    });
  });

  it("drops stored API keys for providers that do not use keys", () => {
    const settings = normalizeAiSettings({
      defaultProviderId: "ollama",
      providers: [
        {
          apiKey: "stale-local-key",
          baseUrl: "http://localhost:11434/v1",
          enabled: true,
          id: "ollama",
          models: [{ enabled: true, id: "llama3.3", name: "Llama" }],
          name: "Ollama",
          type: "ollama"
        }
      ]
    });

    expect(settings.providers.find((provider) => provider.id === "ollama")).toMatchObject({
      apiKey: "",
      baseUrl: "http://localhost:11434/v1"
    });
  });
});
