import process from "node:process";
import { mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import {
  DEFAULT_IMAGE_DIR,
  DEFAULT_PROMPT_DIR,
  DEFAULT_MODEL,
  buildBaseUrl,
  buildDefaultImagePath,
  buildDefaultPromptPath,
  formatApiErrorTable,
  loadAmbientEnv,
  readPromptInput,
  resolveOutput,
  savePrompt,
  slugify,
  mimeFor,
} from "./shared.js";

function printHelp() {
  console.log(`Usage:
  node scripts/generate-chat.js --prompt "A cute baby sea otter" --image out/otter.png

Options:
  --prompt <text>              Prompt text
  --promptfile <path>          Load prompt from a file
  --prompt-output <path>       Save the final prompt to a specific file
  --image <path>               Output image path
  --model <name>               Model override (default: ${DEFAULT_MODEL})
  --json                       Print structured output
  -h, --help                   Show help`);
}

function parseCli(argv) {
  const cfg = {
    prompt: null,
    promptFile: null,
    promptOutput: null,
    imagePath: null,
    model: null,
    json: false,
    help: false,
  };
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === "-h" || arg === "--help") { cfg.help = true; continue; }
    if (arg === "--json") { cfg.json = true; continue; }
    if (arg === "--prompt") { cfg.prompt = argv[++i] || null; if (!cfg.prompt) throw new Error("Missing value for --prompt"); continue; }
    if (arg === "--promptfile") { cfg.promptFile = argv[++i] || null; if (!cfg.promptFile) throw new Error("Missing value for --promptfile"); continue; }
    if (arg === "--prompt-output") { cfg.promptOutput = argv[++i] || null; if (!cfg.promptOutput) throw new Error("Missing value for --prompt-output"); continue; }
    if (arg === "--image") { cfg.imagePath = argv[++i] || null; if (!cfg.imagePath) throw new Error("Missing value for --image"); continue; }
    if (arg === "--model") { cfg.model = argv[++i] || null; if (!cfg.model) throw new Error("Missing value for --model"); continue; }
    throw new Error(`Unknown option: ${arg}`);
  }
  return cfg;
}

function extractImageUrl(content) {
  const mdMatch = content.match(/!\[.*?\]\((https?:\/\/[^\s)]+)\)/);
  if (mdMatch) return mdMatch[1];
  const urlMatch = content.match(/(https?:\/\/[^\s"'<]+\.(?:png|jpg|jpeg|webp|gif))/i);
  if (urlMatch) return urlMatch[1];
  return null;
}

async function downloadImage(url) {
  const res = await fetch(url);
  if (!res.ok) throw new Error(`Failed to download image (${res.status}): ${await res.text().catch(() => "")}`);
  return Buffer.from(await res.arrayBuffer());
}

function buildRequestUrl() {
  return `${buildBaseUrl()}/chat/completions`;
}

function requireApiKey() {
  const key = process.env.OPENAI_API_KEY;
  if (!key) throw new Error("OPENAI_API_KEY is required.");
  return key;
}

async function run() {
  const cfg = parseCli(process.argv.slice(2));
  if (cfg.help) { printHelp(); return; }

  await loadAmbientEnv();
  const model = cfg.model || process.env.OPENAI_IMAGE_MODEL || DEFAULT_MODEL;
  const prompt = await readPromptInput(cfg.prompt, cfg.promptFile);

  const nameHint = slugify(prompt.split(/\s+/).slice(0, 8).join(" "), "generated-image");
  const promptPath = await savePrompt(prompt, cfg.promptOutput, nameHint);
  const outputPath = resolveOutput(cfg.imagePath, buildDefaultImagePath("generate", nameHint));

  const requestBody = {
    model,
    messages: [{ role: "user", content: prompt }],
  };

  const url = buildRequestUrl();
  const apiKey = requireApiKey();
  const res = await fetch(url, {
    method: "POST",
    headers: {
      "Authorization": `Bearer ${apiKey}`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify(requestBody),
  });

  if (!res.ok) {
    const text = await res.text();
    throw new Error(`Chat API error (${res.status}): ${text}`);
  }

  const json = await res.json();
  const content = json?.choices?.[0]?.message?.content;
  if (!content) throw new Error(`No content in response: ${JSON.stringify(json).slice(0, 500)}`);

  const imageUrl = extractImageUrl(content);
  if (!imageUrl) throw new Error(`No image URL found in content: ${content.slice(0, 500)}`);

  const bytes = await downloadImage(imageUrl);
  await mkdir(path.dirname(outputPath), { recursive: true });
  await writeFile(outputPath, bytes);

  if (cfg.json) {
    console.log(JSON.stringify({
      savedImage: outputPath,
      savedPrompt: promptPath,
      model,
      imageUrl,
    }, null, 2));
    return;
  }

  console.log(outputPath);
}

run().catch((error) => {
  const message = error instanceof Error ? error.message : String(error);
  const statusMatch = message.match(/(?:Chat API|Image API|API)\s+error\s*\((\d+)\)/);
  if (statusMatch) {
    formatApiErrorTable(parseInt(statusMatch[1], 10), message);
  } else {
    console.error(message);
  }
  process.exit(1);
});
