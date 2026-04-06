const { invoke } = window.__TAURI__.core;

const FIELD_SCHEMA = [
  { group: "Required", advanced: false, fields: [
    { key: "name",          label: "Name",                type: "text",   required: false, placeholder: "e.g. 2024 Corvette Z06" },
    { key: "weight_lbs",    label: "Weight (lbs)",        type: "number", required: true,  placeholder: "Curb + driver + fuel" },
    { key: "hp_peak",       label: "Peak HP",             type: "number", required: true,  placeholder: "e.g. 480" },
    { key: "driven_wheels", label: "Driven Wheels",       type: "select", required: true,  options: ["RWD","FWD","AWD"] },
  ]},
  { group: "Engine", advanced: true, fields: [
    { key: "tq_peak",  label: "Peak Torque (lb-ft)", type: "number", placeholder: "e.g. 420" },
    { key: "rpm_hp",   label: "Peak HP RPM",         type: "number", placeholder: "e.g. 7000" },
    { key: "rpm_tq",   label: "Peak Torque RPM",     type: "number", placeholder: "e.g. 4600" },
    { key: "redline",  label: "Redline RPM",          type: "number", placeholder: "e.g. 7400" },
  ]},
  { group: "Transmission", advanced: true, fields: [
    { key: "transmission_type", label: "Type",            type: "select", options: ["","auto","manual","dual_clutch","single_speed"] },
    { key: "gears",             label: "Gear Ratios",     type: "text",   placeholder: "e.g. 3.14, 2.32, 1.51, 1.14, 0.87, 0.66", fullWidth: true },
    { key: "final_drive",       label: "Final Drive",     type: "number", placeholder: "e.g. 3.55" },
    { key: "shift_time_s",      label: "Shift Time (s)",  type: "number", placeholder: "e.g. 0.15", step: "0.01" },
  ]},
  { group: "Chassis & Aero", advanced: true, fields: [
    { key: "vehicle_class",       label: "Vehicle Class",       type: "select", options: ["","sports_car","sedan","muscle_car","suv","ev"] },
    { key: "cd",                  label: "Drag Coefficient",    type: "number", placeholder: "e.g. 0.35", step: "0.01" },
    { key: "frontal_area_sqft",   label: "Frontal Area (sqft)", type: "number", placeholder: "e.g. 24.5" },
    { key: "wheelbase_in",        label: "Wheelbase (in)",      type: "number", placeholder: "e.g. 107" },
    { key: "cg_height_in",        label: "CG Height (in)",      type: "number", placeholder: "e.g. 20" },
    { key: "weight_dist_rear_pct",label: "Rear Weight %",       type: "number", placeholder: "e.g. 52" },
    { key: "drivetrain_loss_pct", label: "Drivetrain Loss %",   type: "number", placeholder: "e.g. 15" },
  ]},
  { group: "Tires & Launch", advanced: true, fields: [
    { key: "tire_diameter_in", label: "Tire Diameter (in)",  type: "number", placeholder: "e.g. 27.5" },
    { key: "tire_mu",          label: "Tire Friction Coeff", type: "number", placeholder: "e.g. 1.1", step: "0.01" },
    { key: "launch_rpm",       label: "Launch RPM",          type: "number", placeholder: "e.g. 3500" },
  ]},
];

let racePresets = [];

async function init() {
  buildForm("car-a-form", "a");
  buildForm("car-b-form", "b");

  for (const side of ["a", "b"]) {
    const toggle = document.getElementById(`car-${side}-advanced`);
    const form = document.getElementById(`car-${side}-form`);
    toggle.addEventListener("change", () => {
      form.classList.toggle("show-advanced", toggle.checked);
    });

    document.getElementById(`car-${side}-save`).addEventListener("click", () => handleSave(side));
    document.getElementById(`car-${side}-clear`).addEventListener("click", () => handleClear(side));
  }

  try {
    racePresets = await invoke("get_race_presets");
    buildRaceChips();
  } catch (e) {
    console.error("Failed to load race presets:", e);
  }

  try {
    const cars = await invoke("list_cars");
    populatePresetDropdowns(cars);
  } catch (e) {
    console.error("Failed to list cars:", e);
  }

  try {
    const env = await invoke("get_environment");
    fillEnvironment(env);
  } catch (e) {
    console.error("Failed to load environment:", e);
  }

  document.getElementById("env-save").addEventListener("click", handleSaveEnvironment);
  document.getElementById("compare-btn").addEventListener("click", handleCompare);
}

function buildForm(containerId, prefix) {
  const container = document.getElementById(containerId);
  let html = "";
  for (const group of FIELD_SCHEMA) {
    const advCls = group.advanced ? " field-group-advanced" : "";
    const rowAdvCls = group.advanced ? " field-row-advanced" : "";
    html += `<div class="field-group-title${advCls}">${group.group}</div><div class="field-row${rowAdvCls}">`;
    for (const f of group.fields) {
      const id = `${prefix}-${f.key}`;
      const reqClass = f.required ? " required-field" : "";
      const fullClass = f.fullWidth ? " full-width" : "";
      html += `<div class="field${fullClass}"><label for="${id}">${f.label}</label>`;
      if (f.type === "select") {
        const opts = (f.options || []).map(o =>
          `<option value="${o}">${o || "(auto)"}</option>`
        ).join("");
        html += `<select id="${id}" class="${reqClass}">${opts}</select>`;
      } else {
        const step = f.step ? ` step="${f.step}"` : (f.type === "number" ? ' step="any"' : "");
        html += `<input id="${id}" type="${f.type}" class="${reqClass}" placeholder="${f.placeholder || ""}"${step}>`;
      }
      html += `</div>`;
    }
    html += `</div>`;
  }
  container.innerHTML = html;
}

function buildRaceChips() {
  const wrap = document.getElementById("race-checkboxes");
  wrap.innerHTML = racePresets.map((r, i) => `
    <label class="race-chip selected">
      <input type="checkbox" value="${r.name}" checked data-idx="${i}">
      ${r.name}
    </label>
  `).join("");
  wrap.querySelectorAll("input[type=checkbox]").forEach(cb => {
    cb.addEventListener("change", () => {
      cb.closest(".race-chip").classList.toggle("selected", cb.checked);
    });
  });
}

function populatePresetDropdowns(names) {
  for (const side of ["a", "b"]) {
    const sel = document.getElementById(`car-${side}-preset`);
    for (const n of names) {
      const opt = document.createElement("option");
      opt.value = n;
      opt.textContent = n.replace(/_/g, " ");
      sel.appendChild(opt);
    }
    sel.addEventListener("change", () => loadPreset(side, sel.value));
  }
}

async function loadPreset(side, filename) {
  if (!filename) return;
  try {
    const car = await invoke("load_car_config", { filename });
    fillForm(side, car);
    document.getElementById(`car-${side}-filename`).value = filename;
  } catch (e) {
    console.error("Failed to load car:", e);
  }
}

function fillForm(prefix, car) {
  for (const group of FIELD_SCHEMA) {
    for (const f of group.fields) {
      const el = document.getElementById(`${prefix}-${f.key}`);
      if (!el) continue;
      let val = car[f.key];
      if (val === null || val === undefined) {
        el.value = "";
      } else if (f.key === "gears" && Array.isArray(val)) {
        el.value = val.join(", ");
      } else {
        el.value = val;
      }
    }
  }
}

function readForm(prefix) {
  const car = {};
  for (const group of FIELD_SCHEMA) {
    for (const f of group.fields) {
      const el = document.getElementById(`${prefix}-${f.key}`);
      if (!el) continue;
      const raw = el.value.trim();
      if (raw === "") {
        car[f.key] = null;
        continue;
      }
      if (f.key === "gears") {
        car[f.key] = raw.split(",").map(s => parseFloat(s.trim())).filter(n => !isNaN(n));
        if (car[f.key].length === 0) car[f.key] = null;
      } else if (f.type === "number") {
        const n = parseFloat(raw);
        car[f.key] = isNaN(n) ? null : n;
      } else if (f.type === "select" && raw === "") {
        car[f.key] = null;
      } else {
        car[f.key] = raw;
      }
    }
  }
  return car;
}

function selectedRaceNames() {
  return Array.from(
    document.querySelectorAll("#race-checkboxes input[type=checkbox]:checked")
  ).map(cb => cb.value);
}

function fillEnvironment(env) {
  document.getElementById("env-name").value = env.name || "";
  document.getElementById("env-air-density").value = env.air_density_kgm3 ?? "";
}

function readEnvironment() {
  const name = document.getElementById("env-name").value.trim() || null;
  const raw = document.getElementById("env-air-density").value.trim();
  const air_density_kgm3 = raw ? parseFloat(raw) : null;
  return { name, air_density_kgm3: isNaN(air_density_kgm3) ? null : air_density_kgm3 };
}

async function handleSaveEnvironment() {
  const env = readEnvironment();
  try {
    await invoke("save_environment", { env });
  } catch (e) {
    console.error("Failed to save environment:", e);
    alert("Failed to save environment: " + e);
  }
}

async function handleSave(side) {
  const filenameInput = document.getElementById(`car-${side}-filename`);
  const filename = filenameInput.value.trim().replace(/\.toml$/i, "");
  if (!filename) {
    alert("Enter a config name before saving.");
    filenameInput.focus();
    return;
  }
  const car = readForm(side);
  if (!car.weight_lbs || !car.hp_peak || !car.driven_wheels) {
    alert("At minimum, Weight, Peak HP, and Driven Wheels are required to save.");
    return;
  }
  try {
    await invoke("save_car_config", { filename, car });
    await refreshCarList();
    const sel = document.getElementById(`car-${side}-preset`);
    sel.value = filename;
  } catch (e) {
    console.error("Save failed:", e);
    alert("Save failed: " + e);
  }
}

function handleClear(side) {
  for (const group of FIELD_SCHEMA) {
    for (const f of group.fields) {
      const el = document.getElementById(`${side}-${f.key}`);
      if (!el) continue;
      el.value = "";
    }
  }
  document.getElementById(`car-${side}-filename`).value = "";
  document.getElementById(`car-${side}-preset`).value = "";
}

async function refreshCarList() {
  try {
    const cars = await invoke("list_cars");
    for (const side of ["a", "b"]) {
      const sel = document.getElementById(`car-${side}-preset`);
      const current = sel.value;
      while (sel.options.length > 1) sel.remove(1);
      for (const n of cars) {
        const opt = document.createElement("option");
        opt.value = n;
        opt.textContent = n.replace(/_/g, " ");
        sel.appendChild(opt);
      }
      sel.value = current;
    }
  } catch (e) {
    console.error("Failed to refresh car list:", e);
  }
}

async function handleCompare() {
  const btn = document.getElementById("compare-btn");
  btn.disabled = true;
  btn.textContent = "Simulating...";

  try {
    const carA = readForm("a");
    const carB = readForm("b");

    if (!carA.weight_lbs || !carA.hp_peak || !carA.driven_wheels) {
      alert("Car A needs at least Weight, Peak HP, and Driven Wheels.");
      return;
    }
    if (!carB.weight_lbs || !carB.hp_peak || !carB.driven_wheels) {
      alert("Car B needs at least Weight, Peak HP, and Driven Wheels.");
      return;
    }

    const raceNames = selectedRaceNames();
    const result = await invoke("run_sim", {
      carA: carA,
      carB: carB,
      raceNames: raceNames,
    });

    renderResults(result);
    renderAssumptions(result);
  } catch (e) {
    console.error(e);
    alert("Simulation error: " + e);
  } finally {
    btn.disabled = false;
    btn.textContent = "Compare";
  }
}

function renderResults(result) {
  const sec = document.getElementById("results-section");
  sec.classList.remove("hidden");

  const nameA = result.car_a.name;
  const nameB = result.car_b.name;

  let rows = "";
  for (const race of result.races) {
    const a = race.result_a;
    const b = race.result_b;

    const fmtA = a.completed ? `${a.time_s.toFixed(3)}s @ ${a.final_speed_mph.toFixed(1)} mph` : (a.dnf_reason || "DNF");
    const fmtB = b.completed ? `${b.time_s.toFixed(3)}s @ ${b.final_speed_mph.toFixed(1)} mph` : (b.dnf_reason || "DNF");

    let clsA = "", clsB = "", winnerText = "";
    if (a.completed && b.completed) {
      if (race.winner === nameA) { clsA = "winner-cell"; clsB = "loser-cell"; }
      else if (race.winner === nameB) { clsB = "winner-cell"; clsA = "loser-cell"; }
      winnerText = race.winner
        ? `${race.winner} by ${race.margin_s.toFixed(3)}s`
        : "Tie";
    } else if (a.completed) {
      clsA = "winner-cell"; clsB = "dnf-cell";
      winnerText = nameA;
    } else if (b.completed) {
      clsB = "winner-cell"; clsA = "dnf-cell";
      winnerText = nameB;
    } else {
      clsA = "dnf-cell"; clsB = "dnf-cell";
      winnerText = "Both DNF";
    }

    rows += `<tr>
      <td>${race.race_name}</td>
      <td class="${clsA}">${fmtA}</td>
      <td class="${clsB}">${fmtB}</td>
      <td class="margin-cell">${winnerText}</td>
    </tr>`;
  }

  document.getElementById("results-table-wrap").innerHTML = `
    <table class="results-table">
      <thead><tr>
        <th>Event</th><th>${nameA}</th><th>${nameB}</th><th>Winner</th>
      </tr></thead>
      <tbody>${rows}</tbody>
    </table>`;
}

function renderAssumptions(result) {
  const sec = document.getElementById("assumptions-section");
  sec.classList.remove("hidden");

  const confWrap = document.getElementById("confidence-wrap");
  confWrap.innerHTML =
    confidenceCard(result.car_a.name, result.confidence_a) +
    confidenceCard(result.car_b.name, result.confidence_b);

  const assWrap = document.getElementById("assumptions-wrap");
  assWrap.innerHTML =
    assumptionPanel(result.car_a.name, result.details_a) +
    assumptionPanel(result.car_b.name, result.details_b);
}

function confidenceCard(name, score) {
  const pct = (score * 100).toFixed(0);
  const cls = score >= 0.75 ? "conf-high" : score >= 0.45 ? "conf-med" : "conf-low";
  return `<div class="confidence-card">
    <h3>${name}</h3>
    <div class="confidence-bar-track">
      <div class="confidence-bar-fill ${cls}" style="width:${pct}%"></div>
    </div>
    <span class="confidence-pct">${pct}% of data provided by user</span>
  </div>`;
}

function assumptionPanel(name, details) {
  const items = details.map(d => {
    const cls = d.provided ? "src-provided" : sourceClassFromText(d.source);
    const impactCls = `impact-${d.impact}`;
    return `<li>
      <span>${fieldLabel(d.field)}<span class="impact-badge ${impactCls}">${d.impact}</span></span>
      <span class="${cls}">${d.source}</span>
    </li>`;
  }).join("");
  return `<div><h3 style="font-size:.9rem;margin-bottom:8px">${name}</h3>
    <ul class="assumption-list">${items}</ul></div>`;
}

function sourceClassFromText(src) {
  if (src === "provided") return "src-provided";
  if (src.startsWith("derived")) return "src-derived";
  if (src.startsWith("profile")) return "src-profile";
  return "src-default";
}

function sourceClass(src) {
  if (src === "Provided") return "src-provided";
  if (typeof src === "object") {
    if (src.DerivedFromProvided) return "src-derived";
    if (src.ProfileDefault) return "src-profile";
  }
  return "src-default";
}

function sourceText(src) {
  if (src === "Provided") return "provided";
  if (typeof src === "object") {
    if (src.DerivedFromProvided) return `derived: ${src.DerivedFromProvided.description}`;
    if (src.ProfileDefault) return `profile: ${src.ProfileDefault.profile_name}`;
  }
  return "global default";
}

function fieldLabel(key) {
  return key.replace(/_/g, " ").replace(/\b\w/g, c => c.toUpperCase());
}

document.addEventListener("DOMContentLoaded", init);
