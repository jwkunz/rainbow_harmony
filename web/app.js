import init, {
  render_event_args,
  render_progression,
} from "./pkg/rainbow_harmony_web_wasm.js";

const PRESET_STORAGE_KEY = "rainbow_harmony_presets_v1";

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
const timelineListEl = document.getElementById("timelineList");
const sampleRateEl = document.getElementById("sampleRate");
const progressionModeEl = document.getElementById("progressionMode");
const gapMsEl = document.getElementById("gapMs");
const legatoMsEl = document.getElementById("legatoMs");
const renderPlayButtonEl = document.getElementById("renderPlayButton");
const exportWavButtonEl = document.getElementById("exportWavButton");
const stopButtonEl = document.getElementById("stopButton");
const appendFullStepButtonEl = document.getElementById("appendFullStepButton");
const appendStickyStepButtonEl = document.getElementById("appendStickyStepButton");
const clearProgressionButtonEl = document.getElementById("clearProgressionButton");
const presetNameEl = document.getElementById("presetName");
const presetSelectEl = document.getElementById("presetSelect");
const savePresetButtonEl = document.getElementById("savePresetButton");
const loadPresetButtonEl = document.getElementById("loadPresetButton");
const deletePresetButtonEl = document.getElementById("deletePresetButton");
const presetStatusEl = document.getElementById("presetStatus");
const statusEl = document.getElementById("status");
const renderStatsEl = document.getElementById("renderStats");
const waveformCanvasEl = document.getElementById("waveformCanvas");

let audioContext = null;
let activeSource = null;
let wasmReady = false;
let lastRenderedSamples = [];
let lastRenderedSampleRate = 48_000;

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

function setPresetStatus(message, isError = false) {
  presetStatusEl.textContent = message;
  presetStatusEl.classList.toggle("error", isError);
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

function builderState() {
  return {
    inputType: eventInputTypeEl.value,
    eventValue: eventValueEl.value.trim(),
    durationMode: durationModeEl.value,
    beats: beatsValueEl.value,
    bpm: bpmValueEl.value,
    seconds: secondsValueEl.value,
    gain: gainValueEl.value,
    harmonics: harmonicsValueEl.value,
    harmonicDecay: harmonicDecayValueEl.value,
    phaseModRate: phaseModRateValueEl.value,
    phaseModDepth: phaseModDepthValueEl.value,
    ampTremoloRate: ampTremoloRateValueEl.value,
    ampTremoloDepth: ampTremoloDepthValueEl.value,
    vibratoRate: vibratoRateValueEl.value,
    vibratoDepth: vibratoDepthValueEl.value,
    noiseDepth: noiseDepthValueEl.value,
    seed: seedValueEl.value,
  };
}

function applyBuilderState(state) {
  eventInputTypeEl.value = state.inputType ?? "chord";
  eventValueEl.value = state.eventValue ?? "Cmaj7";
  durationModeEl.value = state.durationMode ?? "beats";
  beatsValueEl.value = state.beats ?? "2";
  bpmValueEl.value = state.bpm ?? "120";
  secondsValueEl.value = state.seconds ?? "2.0";
  gainValueEl.value = state.gain ?? "1.0";
  harmonicsValueEl.value = state.harmonics ?? "33";
  harmonicDecayValueEl.value = state.harmonicDecay ?? "0.8";
  phaseModRateValueEl.value = state.phaseModRate ?? "1000";
  phaseModDepthValueEl.value = state.phaseModDepth ?? "0.05";
  ampTremoloRateValueEl.value = state.ampTremoloRate ?? "3";
  ampTremoloDepthValueEl.value = state.ampTremoloDepth ?? "0.05";
  vibratoRateValueEl.value = state.vibratoRate ?? "5";
  vibratoDepthValueEl.value = state.vibratoDepth ?? "0.25";
  noiseDepthValueEl.value = state.noiseDepth ?? "0.02";
  seedValueEl.value = state.seed ?? "0";
  updateEventPreview();
}

function loadPresets() {
  try {
    return JSON.parse(localStorage.getItem(PRESET_STORAGE_KEY) ?? "[]");
  } catch {
    return [];
  }
}

function savePresets(presets) {
  localStorage.setItem(PRESET_STORAGE_KEY, JSON.stringify(presets));
}

function renderPresetOptions() {
  const presets = loadPresets();
  const current = presetSelectEl.value;
  presetSelectEl.innerHTML = "";

  if (!presets.length) {
    const option = document.createElement("option");
    option.value = "";
    option.textContent = "No saved presets";
    presetSelectEl.append(option);
    presetSelectEl.disabled = true;
    loadPresetButtonEl.disabled = true;
    deletePresetButtonEl.disabled = true;
    return;
  }

  presetSelectEl.disabled = false;
  loadPresetButtonEl.disabled = false;
  deletePresetButtonEl.disabled = false;

  for (const preset of presets) {
    const option = document.createElement("option");
    option.value = preset.name;
    option.textContent = preset.name;
    presetSelectEl.append(option);
  }

  presetSelectEl.value = presets.some((preset) => preset.name === current)
    ? current
    : presets[0].name;
}

function saveCurrentPreset() {
  const name = presetNameEl.value.trim();
  if (!name) {
    setPresetStatus("Preset name is required.", true);
    return;
  }

  const presets = loadPresets().filter((preset) => preset.name !== name);
  presets.push({ name, state: builderState() });
  presets.sort((left, right) => left.name.localeCompare(right.name));
  savePresets(presets);
  renderPresetOptions();
  presetSelectEl.value = name;
  setPresetStatus(`Saved preset "${name}".`);
}

function loadSelectedPreset() {
  const name = presetSelectEl.value;
  const preset = loadPresets().find((entry) => entry.name === name);
  if (!preset) {
    setPresetStatus("Select a preset first.", true);
    return;
  }

  presetNameEl.value = preset.name;
  applyBuilderState(preset.state);
  setPresetStatus(`Loaded preset "${preset.name}".`);
}

function deleteSelectedPreset() {
  const name = presetSelectEl.value;
  if (!name) {
    setPresetStatus("Select a preset first.", true);
    return;
  }

  savePresets(loadPresets().filter((preset) => preset.name !== name));
  renderPresetOptions();
  setPresetStatus(`Deleted preset "${name}".`);
}

function buildEventFlagsFromControls() {
  const parts = [];
  const state = builderState();

  if (!state.eventValue) {
    return "";
  }

  parts.push(`--${state.inputType}`, state.eventValue);

  if (state.durationMode === "beats") {
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

function tokenizeFlags(text) {
  return text.trim().split(/\s+/).filter(Boolean);
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

function segmentSummary(segment) {
  const map = eventFlagsToMap(segment);
  const input = map.get("--notes") ?? map.get("--chord") ?? "(missing input)";
  const duration = map.has("--seconds")
    ? `${map.get("--seconds")}s`
    : `${map.get("--beats") ?? "?"} beats @ ${map.get("--bpm") ?? "?"} bpm`;

  return `${input} · ${duration}`;
}

function progressionSegments() {
  return progressionInputEl.value
    .split("=>")
    .map((segment) => segment.trim())
    .filter(Boolean);
}

function setProgressionSegments(segments) {
  progressionInputEl.value = segments.join(" => ");
  updateProgressionBuilderState();
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
  if (current.get(inputFlag)) {
    sticky.push(inputFlag, current.get(inputFlag));
  }

  if (current.has("--seconds")) {
    sticky.push("--seconds", current.get("--seconds"));
  } else if (current.has("--beats")) {
    sticky.push("--beats", current.get("--beats"));
    sticky.push("--bpm", current.get("--bpm"));
  }

  for (const [flag, value] of current.entries()) {
    if (["--notes", "--chord", "--beats", "--bpm", "--seconds"].includes(flag)) {
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

  const segments = progressionSegments();
  segments.push(trimmed);
  setProgressionSegments(segments);
}

function loadSegmentIntoBuilder(segment) {
  const map = eventFlagsToMap(segment);
  if (map.has("--notes")) {
    eventInputTypeEl.value = "notes";
    eventValueEl.value = map.get("--notes");
  } else if (map.has("--chord")) {
    eventInputTypeEl.value = "chord";
    eventValueEl.value = map.get("--chord");
  }

  if (map.has("--seconds")) {
    durationModeEl.value = "seconds";
    secondsValueEl.value = map.get("--seconds");
  } else {
    durationModeEl.value = "beats";
    beatsValueEl.value = map.get("--beats") ?? beatsValueEl.value;
    bpmValueEl.value = map.get("--bpm") ?? bpmValueEl.value;
  }

  gainValueEl.value = map.get("--gain") ?? gainValueEl.value;
  harmonicsValueEl.value = map.get("--harmonics") ?? harmonicsValueEl.value;
  harmonicDecayValueEl.value = map.get("--harmonic-decay") ?? harmonicDecayValueEl.value;
  phaseModRateValueEl.value = map.get("--phase-mod-rate") ?? phaseModRateValueEl.value;
  phaseModDepthValueEl.value = map.get("--phase-mod-depth") ?? phaseModDepthValueEl.value;
  ampTremoloRateValueEl.value = map.get("--amp-tremolo-rate") ?? ampTremoloRateValueEl.value;
  ampTremoloDepthValueEl.value =
    map.get("--amp-tremolo-depth") ?? ampTremoloDepthValueEl.value;
  vibratoRateValueEl.value = map.get("--vibrato-rate") ?? vibratoRateValueEl.value;
  vibratoDepthValueEl.value =
    map.get("--vibrato-depth-cents") ?? vibratoDepthValueEl.value;
  noiseDepthValueEl.value = map.get("--noise-depth") ?? noiseDepthValueEl.value;
  seedValueEl.value = map.get("--seed") ?? seedValueEl.value;

  updateEventPreview();
}

function moveSegment(index, delta) {
  const segments = progressionSegments();
  const target = index + delta;
  if (target < 0 || target >= segments.length) {
    return;
  }
  [segments[index], segments[target]] = [segments[target], segments[index]];
  setProgressionSegments(segments);
}

function duplicateSegment(index) {
  const segments = progressionSegments();
  segments.splice(index + 1, 0, segments[index]);
  setProgressionSegments(segments);
}

function deleteSegment(index) {
  const segments = progressionSegments();
  segments.splice(index, 1);
  setProgressionSegments(segments);
}

function replaceSegment(index, text) {
  const segments = progressionSegments();
  segments[index] = text.trim();
  setProgressionSegments(segments);
}

function renderTimeline() {
  const segments = progressionSegments();
  timelineListEl.innerHTML = "";

  if (!segments.length) {
    const empty = document.createElement("p");
    empty.className = "muted";
    empty.textContent = "Timeline is empty.";
    timelineListEl.append(empty);
    return;
  }

  for (const [index, segment] of segments.entries()) {
    const item = document.createElement("article");
    item.className = "timeline-item";
    item.dataset.index = String(index);
    item.innerHTML = `
      <div class="timeline-copy">
        <p class="timeline-step">Step ${index + 1}</p>
        <p class="timeline-summary">${segmentSummary(segment)}</p>
        <p class="timeline-flags">${segment}</p>
      </div>
      <div class="timeline-actions">
        <button type="button" data-action="load">Load</button>
        <button type="button" data-action="replace">Replace</button>
        <button type="button" data-action="dup">Duplicate</button>
        <button type="button" data-action="up" class="secondary">Up</button>
        <button type="button" data-action="down" class="secondary">Down</button>
        <button type="button" data-action="delete" class="secondary">Delete</button>
      </div>
    `;
    timelineListEl.append(item);
  }
}

function updateProgressionBuilderState() {
  const stickyFlags = stickyFlagsForCurrentEvent();
  stickyPreviewEl.value = stickyFlags || eventInputEl.value.trim();

  const segmentCount = progressionSegments().length;
  progressionSummaryEl.textContent =
    segmentCount === 0
      ? "No progression steps yet."
      : `${segmentCount} step${segmentCount === 1 ? "" : "s"} in progression.`;
  renderTimeline();
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

function bufferToWavBlob(samples, sampleRate) {
  const pcm = new Int16Array(samples.length);
  for (let index = 0; index < samples.length; index += 1) {
    const clamped = Math.max(-1, Math.min(1, samples[index]));
    pcm[index] = clamped < 0 ? clamped * 0x8000 : clamped * 0x7fff;
  }

  const buffer = new ArrayBuffer(44 + pcm.length * 2);
  const view = new DataView(buffer);
  const writeString = (offset, value) => {
    for (let index = 0; index < value.length; index += 1) {
      view.setUint8(offset + index, value.charCodeAt(index));
    }
  };

  writeString(0, "RIFF");
  view.setUint32(4, 36 + pcm.length * 2, true);
  writeString(8, "WAVE");
  writeString(12, "fmt ");
  view.setUint32(16, 16, true);
  view.setUint16(20, 1, true);
  view.setUint16(22, 1, true);
  view.setUint32(24, sampleRate, true);
  view.setUint32(28, sampleRate * 2, true);
  view.setUint16(32, 2, true);
  view.setUint16(34, 16, true);
  writeString(36, "data");
  view.setUint32(40, pcm.length * 2, true);

  let offset = 44;
  for (const sample of pcm) {
    view.setInt16(offset, sample, true);
    offset += 2;
  }

  return new Blob([buffer], { type: "audio/wav" });
}

function downloadBlob(blob, filename) {
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = filename;
  anchor.click();
  URL.revokeObjectURL(url);
}

function renderAndMaybePlay(playAudio) {
  if (!wasmReady) {
    throw new Error("Wasm package is still loading.");
  }

  const result = renderCurrentSelection();
  const samples = result.samples();
  lastRenderedSamples = samples;
  lastRenderedSampleRate = result.sample_rate;
  drawWaveform(samples);
  renderStatsEl.textContent =
    `${result.frame_count.toLocaleString()} frames at ${result.sample_rate.toLocaleString()} Hz, ` +
    `${result.duration_seconds.toFixed(2)} seconds`;

  if (!playAudio) {
    return Promise.resolve(result);
  }

  const context = getAudioContext();
  return context.resume().then(() => {
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
    return result;
  });
}

async function playCurrentSelection() {
  try {
    stopPlayback();
    await renderAndMaybePlay(true);
    setStatus("Playing.");
  } catch (error) {
    setStatus(error instanceof Error ? error.message : String(error), true);
    if (!lastRenderedSamples.length) {
      drawWaveform([]);
    }
  }
}

async function exportCurrentSelectionToWav() {
  try {
    stopPlayback();
    await renderAndMaybePlay(false);
    const blob = bufferToWavBlob(lastRenderedSamples, lastRenderedSampleRate);
    const filename = modeEl.value === "progression" ? "progression.wav" : "event.wav";
    downloadBlob(blob, filename);
    setStatus(`Exported ${filename}.`);
  } catch (error) {
    setStatus(error instanceof Error ? error.message : String(error), true);
  }
}

renderPlayButtonEl.addEventListener("click", () => {
  void playCurrentSelection();
});

exportWavButtonEl.addEventListener("click", () => {
  void exportCurrentSelectionToWav();
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

savePresetButtonEl.addEventListener("click", saveCurrentPreset);
loadPresetButtonEl.addEventListener("click", loadSelectedPreset);
deletePresetButtonEl.addEventListener("click", deleteSelectedPreset);

timelineListEl.addEventListener("click", (event) => {
  const button = event.target.closest("button[data-action]");
  if (!button) {
    return;
  }

  const item = button.closest(".timeline-item");
  if (!item) {
    return;
  }

  const index = Number.parseInt(item.dataset.index, 10);
  const segments = progressionSegments();
  const segment = segments[index];
  if (segment === undefined) {
    return;
  }

  switch (button.dataset.action) {
    case "load":
      loadSegmentIntoBuilder(segment);
      break;
    case "replace":
      replaceSegment(index, eventInputEl.value);
      break;
    case "dup":
      duplicateSegment(index);
      break;
    case "up":
      moveSegment(index, -1);
      break;
    case "down":
      moveSegment(index, 1);
      break;
    case "delete":
      deleteSegment(index);
      break;
    default:
      break;
  }
});

async function bootstrap() {
  try {
    await init();
    wasmReady = true;
    renderPresetOptions();
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
