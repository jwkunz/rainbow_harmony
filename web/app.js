import init, {
  render_event_args,
  render_progression,
} from "./pkg/rainbow_harmony_web_wasm.js";

const modeEl = document.getElementById("mode");
const eventInputEl = document.getElementById("eventInput");
const progressionInputEl = document.getElementById("progressionInput");
const sampleRateEl = document.getElementById("sampleRate");
const progressionModeEl = document.getElementById("progressionMode");
const gapMsEl = document.getElementById("gapMs");
const legatoMsEl = document.getElementById("legatoMs");
const renderPlayButtonEl = document.getElementById("renderPlayButton");
const stopButtonEl = document.getElementById("stopButton");
const statusEl = document.getElementById("status");
const renderStatsEl = document.getElementById("renderStats");

let audioContext = null;
let activeSource = null;
let wasmReady = false;

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
  progressionInputEl.disabled = !progressionMode;
  progressionModeEl.disabled = !progressionMode;
  gapMsEl.disabled = !progressionMode;
  legatoMsEl.disabled = !progressionMode;
  eventInputEl.disabled = progressionMode;
});

async function bootstrap() {
  try {
    await init();
    wasmReady = true;
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
