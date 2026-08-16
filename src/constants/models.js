export const OPENROUTER_ID_MAP = {
  "InclusionAi": "inclusionai/ling-3.0-tiny:free",
  "Poolside: Laguna S 2.1": "poolside/laguna-s-2.1:free",
  "Poolside: Laguna XS 2.1": "poolside/laguna-xs-2.1:free",
  "Cohere North mini": "cohere/north-mini-code:free",
  "Nvidia Nemotron 3.5 (content safety)": "nvidia/nemotron-3.5-content-safety:free",
  "Nvidia Nemotron 3 Ultra": "nvidia/nemotron-3-ultra-550b-a55b:free",
  "Nvidia Nemotron 3 Nano": "nvidia/nemotron-3-nano-30b-a3b:free",
  "Nvidia Nemotron Nano (9B)": "nvidia/nemotron-nano-9b-v2:free",
  "Nvidia Nemotron 3 Nano Omni": "nvidia/nemotron-3-nano-omni-30b-a3b-reasoning:free",
  "Nvidia Nemotron 3 Super": "nvidia/nemotron-3-super-120b-a12b:free",
  "Google Gemma 4": "google/gemma-4-31b-it:free",
  "Google Gemma 4 (A4B)": "google/gemma-4-26b-a4b-it:free",
  "OpenAI GPT OSS": "openai/gpt-oss-20b:free",
};

// Models where thinking/reasoning can be enabled
export const THINKING_SUPPORTED = new Set([
  "Nvidia Nemotron 3 Nano Omni",
  "Google Gemma 4",
  "Google Gemma 4 (A4B)",
]);

// Models that REQUIRE thinking (will be auto-enabled)
export const THINKING_MANDATORY = new Set([
  // Add model names here if any require mandatory thinking
  // e.g. "DeepSeek R1",
]);

// Check if a model supports, requires, or forbids thinking
export function getThinkingStatus(modelName) {
  if (THINKING_MANDATORY.has(modelName)) return "mandatory";
  if (THINKING_SUPPORTED.has(modelName)) return "supported";
  return "unsupported";
}
