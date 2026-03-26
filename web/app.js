import init, {
  render_event_args,
  render_progression,
} from "./pkg/rainbow_harmony_web_wasm.js";

const modeEl = document.getElementById("mode");
const eventBuilderEl = document.getElementById("eventBuilder");
const progressionPanelEl = document.getElementById("progressionPanel");
const eventInputTypeEl = document.getElementById("eventInputType");
const eventValueEl = document.getElementById("eventValue");
const durationModeEl = document.getElementById("durationMode");
const beatsValueEl = document.getElementById("beatsValue");
const bpmValueEl = document.getElementById("bpmValue");
const secondsValueEl = document.getElementById("secondsValue");
const gainValueEl = document.getElementById("gainValue");
const harmonicsValueEl = document.getElementById("harmonicsValue");
const harmonicDecayValueEl = document.getElementById("harmonicDecayValue");
const phaseModRateValueEl = document.getElementById("phaseModRateValue");
const phaseModDepthValueEl = document.getElementById("phaseModDepthValue");
const ampTremoloRateValueEl = document.getElementById("ampTremoloRateValue");
const ampTremoloDepthValueEl = document.getElementById("ampTremoloDepthValue");
const vibratoRateValueEl = document.getElementById("vibratoRateValue");
const vibratoDepthValueEl = document.getElementById("vibratoDepthValue");
const noiseDepthValueEl = document.getElementById("noiseDepthValue");
const seedValueEl = document.getElementById("seedValue");
const eventInputEl = document.getElementById("eventInput");
const stickyPreviewEl = document.getElementById("stickyPreview");
const progressionInputEl = document.getElementById("progressionInput");
const progressionSummaryEl = document.getElementById("progressionSummary");
const sampleRateEl = document.getElementById("sampleRate");
const progressionModeEl = document.getElementById("progressionMode");
const gapMsEl = document.getElementById("gapMs");
const legatoMsEl = document.getElementById("legatoMs");
const renderPlayButtonEl = document.getElementById("renderPlayButton");
const stopButtonEl = document.getElementById("stopButton");
const appendFullStepButtonEl = document.getElementById("appendFullStepButton");
const appendStickyStepButtonEl = document.getElementById("appendStickyStepButton");
const clearProgressionButtonEl = document.getElementById("clearProgressionButton");
const statusEl = document.getElementById("status");
const renderStatsEl = document.getElementById("renderStats");
const waveformCanvasEl = document.getElementById("waveformCanvas");

let audioContext = null;
let activeSource = null;
let wasmReady = false;
let lastRenderedSamples = [];

const builderFields = [
  eventInputTypeEl,
  eventValueEl,
  durationModeEl,
  beatsValueEl,
  bpmValueEl,
  secondsValueEl,
  gainValueEl,
  harmonicsValueEl,
  harmonicDecayValueEl,
  phaseModRateValueEl,
  phaseModDepthValueEl,
  ampTremoloRateValueEl,
  ampTremoloDepthValueEl,
  vibratoRateValueEl,
  vibratoDepthValueEl,
  noiseDepthValueEl,
  seedValueEl,
];

function setStatus(message, isError = false) {
  statusEl.textContent = message;
  statusEl.classList.toggle("error", isError);
}

function getAudioContext() {
  if (!audioContext) {
    audioContext = new AudioContext();
  }
  return audioContext;
}

function stopPlayback() {
  if (activeSource) {
    activeSource.stop();
    activeSource.disconnect();
    activeSource = null;
  }
}

function parseIntegerField(element, fieldName) {
  const value = Number.parseInt(element.value, 10);
  if (!Number.isFinite(value) || value < 0) {
    throw new Error(`Invalid ${fieldName}`);
  }
  return value;
}

function parseNumberField(element, fieldName) {
  const value = Number.parseFloat(element.value);
  if (!Number.isFinite(value)) {
    throw new Error(`Invalid ${fieldName}`);
  }
  return value;
}

function updateDurationFieldState() {
  const beatsMode = durationModeEl.value === "beats";
  beatsValueEl.disabled = !beatsMode;
  bpmValueEl.disabled = !beatsMode;
  secondsValueEl.disabled = beatsMode;
}

function formatNumber(value) {
  return Number(value).toString();
}

function buildEventFlagsFromControls() {
  const parts = [];
  const inputType = eventInputTypeEl.value;
  const eventValue = eventValueEl.value.trim();

  if (!eventValue) {
    return "";
  }

  parts.push(`--${inputType}`, eventValue);

  if (durationModeEl.value === "beats") {
    parts.push("--beats", formatNumber(parseIntegerField(beatsValueEl, "beats")));
    parts.push("--bpm", formatNumber(parseIntegerField(bpmValueEl, "bpm")));
  } else {
    parts.push("--seconds", formatNumber(parseNumberField(secondsValueEl, "seconds")));
  }

  parts.push("--gain", formatNumber(parseNumberField(gainValueEl, "gain")));
  parts.push("--harmonics", formatNumber(parseIntegerField(harmonicsValueEl, "harmonics")));
  parts.push(
    "--harmonic-decay",
    formatNumber(parseNumberField(harmonicDecayValueEl, "harmonic decay")),
  );
  parts.push(
    "--phase-mod-rate",
    formatNumber(parseNumberField(phaseModRateValueEl, "phase modulation rate")),
  );
  parts.push(
    "--phase-mod-depth",
    formatNumber(parseNumberField(phaseModDepthValueEl, "phase modulation depth")),
  );
  parts.push(
    "--amp-tremolo-rate",
    formatNumber(parseNumberField(ampTremoloRateValueEl, "amplitude tremolo rate")),
  );
  parts.push(
    "--amp-tremolo-depth",
    formatNumber(parseNumberField(ampTremoloDepthValueEl, "amplitude tremolo depth")),
  );
  parts.push(
    "--vibrato-rate",
    formatNumber(parseNumberField(vibratoRateValueEl, "vibrato rate")),
  );
  parts.push(
    "--vibrato-depth-cents",
    formatNumber(parseNumberField(vibratoDepthValueEl, "vibrato depth")),
  );
  parts.push(
    "--noise-depth",
    formatNumber(parseNumberField(noiseDepthValueEl, "noise depth")),
  );
  parts.push("--seed", formatNumber(parseIntegerField(seedValueEl, "seed")));

  return parts.join(" ");
}

function updateEventPreview() {
  try {
    updateDurationFieldState();
    eventInputEl.value = buildEventFlagsFromControls();
    eventInputEl.classList.remove("invalid");
    updateProgressionBuilderState();
  } catch {
    eventInputEl.classList.add("invalid");
  }
}

function tokenizeFlags(text) {
  return text
    .trim()
    .split(/\s+/)
    .filter(Boolean);
}

function eventFlagsToMap(text) {
  const tokens = tokenizeFlags(text);
  const map = new Map();

  for (let index = 0; index < tokens.length; index += 2) {
    const flag = tokens[index];
    const value = tokens[index + 1];
    if (!flag || !flag.startsWith("--") || value === undefined) {
      break;
    }
    map.set(flag, value);
  }

  return map;
}

function progressionSegments() {
  return progressionInputEl.value
    .split("=>")
    .map((segment) => segment.trim())
    .filter(Boolean);
}

function effectiveProgressionState() {
  const state = new Map();
  for (const segment of progressionSegments()) {
    const segmentMap = eventFlagsToMap(segment);
    for (const [flag, value] of segmentMap.entries()) {
      state.set(flag, value);
    }
  }
  return state;
}

function removeFlagPair(parts, flag) {
  const index = parts.indexOf(flag);
  if (index >= 0) {
    parts.splice(index, 2);
  }
}

function stickyFlagsForCurrentEvent() {
  const current = eventFlagsToMap(eventInputEl.value);
  const previous = effectiveProgressionState();
  const sticky = [];

  const inputFlag = current.has("--notes") ? "--notes" : "--chord";
  sticky.push(inputFlag, current.get(inputFlag));

  if (current.has("--seconds")) {
    sticky.push("--seconds", current.get("--seconds"));
  } else {
    sticky.push("--beats", current.get("--beats"));
    sticky.push("--bpm", current.get("--bpm"));
  }

  for (const [flag, value] of current.entries()) {
    if (
      flag === "--notes" ||
      flag === "--chord" ||
      flag === "--beats" ||
      flag === "--bpm" ||
      flag === "--seconds"
    ) {
      continue;
    }

    if (previous.get(flag) !== value) {
      sticky.push(flag, value);
    }
  }

  if (previous.get(inputFlag) === current.get(inputFlag)) {
    removeFlagPair(sticky, inputFlag);
  }

  if (current.has("--seconds") && previous.get("--seconds") === current.get("--seconds")) {
    removeFlagPair(sticky, "--seconds");
  }

  if (current.has("--beats")) {
    if (previous.get("--beats") === current.get("--beats")) {
      removeFlagPair(sticky, "--beats");
    }
    if (previous.get("--bpm") === current.get("--bpm")) {
      removeFlagPair(sticky, "--bpm");
    }
  }

  return sticky.join(" ").trim();
}

function appendSegment(segment) {
  const trimmed = segment.trim();
  if (!trimmed) {
    return;
  }

  progressionInputEl.value = progressionSegments().length
    ? `${progressionInputEl.value.trim()} => ${trimmed}`
    : trimmed;
  updateProgressionBuilderState();
}

function updateProgressionBuilderState() {
  const stickyFlags = stickyFlagsForCurrentEvent();
  stickyPreviewEl.value = stickyFlags || eventInputEl.value.trim();

  const segmentCount = progressionSegments().length;
  progressionSummaryEl.textContent =
    segmentCount === 0
      ? "No progression steps yet."
      : `${segmentCount} step${segmentCount === 1 ? "" : "s"} in progression.`;
}

function renderCurrentSelection() {
  const sampleRate = parseIntegerField(sampleRateEl, "sample rate");

  if (modeEl.value === "event") {
    return render_event_args(eventInputEl.value.trim(), sampleRate);
  }

  return render_progression(
    progressionInputEl.value.trim(),
    sampleRate,
    progressionModeEl.value,
    parseIntegerField(gapMsEl, "gap"),
    parseIntegerField(legatoMsEl, "legato"),
  );
}

function drawWaveform(samples) {
  const context = waveformCanvasEl.getContext("2d");
  const width = waveformCanvasEl.width;
  const height = waveformCanvasEl.height;
  const midline = height / 2;

  context.clearRect(0, 0, width, height);
  context.fillStyle = "#fffaf2";
  context.fillRect(0, 0, width, height);

  context.strokeStyle = "rgba(31, 42, 33, 0.14)";
  context.lineWidth = 1;
  context.beginPath();
  context.moveTo(0, midline);
  context.lineTo(width, midline);
  context.stroke();

  if (!samples.length) {
    return;
  }

  const bucketSize = Math.max(1, Math.floor(samples.length / width));
  context.strokeStyle = "#0f766e";
  context.lineWidth = 1.5;
  context.beginPath();

  for (let x = 0; x < width; x += 1) {
    const start = x * bucketSize;
    const end = Math.min(samples.length, start + bucketSize);
    let peak = 0;

    for (let index = start; index < end; index += 1) {
      peak = Math.max(peak, Math.abs(samples[index]));
    }

    const y = midline - peak * (height * 0.42);
    if (x === 0) {
      context.moveTo(x, y);
    } else {
      context.lineTo(x, y);
    }
  }

  for (let x = width - 1; x >= 0; x -= 1) {
    const start = x * bucketSize;
    const end = Math.min(samples.length, start + bucketSize);
    let trough = 0;

    for (let index = start; index < end; index += 1) {
      trough = Math.min(trough, samples[index]);
    }

    const y = midline - trough * (height * 0.42);
    context.lineTo(x, y);
  }

  context.closePath();
  context.fillStyle = "rgba(15, 118, 110, 0.16)";
  context.fill();
  context.stroke();
}

async function playCurrentSelection() {
  if (!wasmReady) {
    setStatus("Wasm package is still loading.", true);
    return;
  }

  try {
    stopPlayback();

    const result = renderCurrentSelection();
    const context = getAudioContext();
    await context.resume();

    const samples = result.samples();
    lastRenderedSamples = samples;
    drawWaveform(samples);
    const audioBuffer = context.createBuffer(1, samples.length, result.sample_rate);
    audioBuffer.copyToChannel(Float32Array.from(samples), 0);

    const source = context.createBufferSource();
    source.buffer = audioBuffer;
    source.connect(context.destination);
    source.onended = () => {
      if (activeSource === source) {
        activeSource = null;
      }
    };
    source.start();
    activeSource = source;

    setStatus("Playing.");
    renderStatsEl.textContent =
      `${result.frame_count.toLocaleString()} frames at ${result.sample_rate.toLocaleString()} Hz, ` +
      `${result.duration_seconds.toFixed(2)} seconds`;
  } catch (error) {
    setStatus(error instanceof Error ? error.message : String(error), true);
    renderStatsEl.textContent = "";
    if (!lastRenderedSamples.length) {
      drawWaveform([]);
    }
  }
}

renderPlayButtonEl.addEventListener("click", () => {
  void playCurrentSelection();
});

stopButtonEl.addEventListener("click", () => {
  stopPlayback();
  setStatus("Stopped.");
});

modeEl.addEventListener("change", () => {
  const progressionMode = modeEl.value === "progression";
  progressionPanelEl.hidden = !progressionMode;
  progressionInputEl.disabled = !progressionMode;
  progressionModeEl.disabled = !progressionMode;
  gapMsEl.disabled = !progressionMode;
  legatoMsEl.disabled = !progressionMode;
  appendFullStepButtonEl.disabled = !progressionMode;
  appendStickyStepButtonEl.disabled = !progressionMode;
  clearProgressionButtonEl.disabled = !progressionMode;
  stickyPreviewEl.disabled = !progressionMode;
});

for (const field of builderFields) {
  field.addEventListener("input", updateEventPreview);
  field.addEventListener("change", updateEventPreview);
}

progressionInputEl.addEventListener("input", updateProgressionBuilderState);

appendFullStepButtonEl.addEventListener("click", () => {
  appendSegment(eventInputEl.value);
});

appendStickyStepButtonEl.addEventListener("click", () => {
  appendSegment(stickyPreviewEl.value);
});

clearProgressionButtonEl.addEventListener("click", () => {
  progressionInputEl.value = "";
  updateProgressionBuilderState();
});

async function bootstrap() {
  try {
    await init();
    wasmReady = true;
    updateEventPreview();
    updateProgressionBuilderState();
    drawWaveform([]);
    modeEl.dispatchEvent(new Event("change"));
    setStatus("Wasm package loaded. Ready to render.");
  } catch (error) {
    setStatus(
      `Failed to load wasm package. Run ./scripts/build_wasm.sh first. ${error}`,
      true,
    );
  }
}

void bootstrap();
