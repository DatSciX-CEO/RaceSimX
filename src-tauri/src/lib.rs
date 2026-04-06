use sim_core::config::ProfileConfig;
use sim_core::confidence::FieldDetail;
use sim_core::validation::ValidationWarning;
use sim_core::*;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{Manager, State};

struct AppState {
    config_dir: PathBuf,
    profiles: ProfileConfig,
    environment: EnvironmentInput,
    races: Vec<RacePreset>,
}

fn find_config_dir(app: &tauri::App) -> PathBuf {
    let cwd_config = std::env::current_dir().unwrap_or_default().join("config");
    if cwd_config.exists() {
        return cwd_config;
    }

    if let Ok(resource) = app.path().resource_dir() {
        let p = resource.join("config");
        if p.exists() {
            return p;
        }
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let p = dir.join("config");
            if p.exists() {
                return p;
            }
        }
    }

    cwd_config
}

#[tauri::command]
fn list_cars(state: State<'_, Mutex<AppState>>) -> Result<Vec<String>, String> {
    let st = state.lock().unwrap();
    list_car_files(&st.config_dir.join("cars"))
}

#[tauri::command]
fn load_car_config(filename: String, state: State<'_, Mutex<AppState>>) -> Result<CarInput, String> {
    let st = state.lock().unwrap();
    load_car(&st.config_dir.join("cars").join(format!("{filename}.toml")))
}

#[tauri::command]
fn save_car_config(filename: String, car: CarInput, state: State<'_, Mutex<AppState>>) -> Result<(), String> {
    let st = state.lock().unwrap();
    let path = st.config_dir.join("cars").join(format!("{filename}.toml"));
    let content = toml::to_string_pretty(&car).map_err(|e| format!("Serialize error: {e}"))?;
    std::fs::write(&path, content).map_err(|e| format!("Write error: {e}"))
}

#[tauri::command]
fn get_race_presets(state: State<'_, Mutex<AppState>>) -> Vec<RacePreset> {
    let st = state.lock().unwrap();
    st.races.clone()
}

#[tauri::command]
fn get_environment(state: State<'_, Mutex<AppState>>) -> EnvironmentInput {
    let st = state.lock().unwrap();
    st.environment.clone()
}

#[tauri::command]
fn save_environment(env: EnvironmentInput, state: State<'_, Mutex<AppState>>) -> Result<(), String> {
    let st = state.lock().unwrap();
    let path = st.config_dir.join("environment").join("standard.toml");
    let content = toml::to_string_pretty(&env).map_err(|e| format!("Serialize error: {e}"))?;
    std::fs::write(&path, content).map_err(|e| format!("Write error: {e}"))?;
    drop(st);
    let mut st = state.lock().unwrap();
    st.environment = env;
    Ok(())
}

#[tauri::command]
fn validate_car(car: CarInput) -> Vec<ValidationWarning> {
    validate_car_input(&car)
}

#[tauri::command]
fn resolve_and_detail(
    car: CarInput,
    state: State<'_, Mutex<AppState>>,
) -> Result<(ResolvedCar, Vec<FieldDetail>, f64), String> {
    let st = state.lock().unwrap();
    let resolved = resolve_car(&car, &st.profiles)?;
    let details = field_details(&resolved);
    let conf = car_confidence(&resolved);
    Ok((resolved, details, conf))
}

#[tauri::command]
fn run_sim(
    car_a: CarInput,
    car_b: CarInput,
    race_names: Vec<String>,
    state: State<'_, Mutex<AppState>>,
) -> Result<ComparisonResult, String> {
    let st = state.lock().unwrap();
    let resolved_a = resolve_car(&car_a, &st.profiles)?;
    let resolved_b = resolve_car(&car_b, &st.profiles)?;

    let selected: Vec<RacePreset> = if race_names.is_empty() {
        st.races.clone()
    } else {
        st.races
            .iter()
            .filter(|r| race_names.contains(&r.name))
            .cloned()
            .collect()
    };

    Ok(run_comparison(
        &resolved_a,
        &resolved_b,
        &st.environment,
        &selected,
        &st.profiles,
    ))
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let config_dir = find_config_dir(app);

            let profiles = load_profiles(&config_dir.join("profiles").join("defaults.toml"))
                .map_err(|e| format!("Profiles: {e}"))?;
            let environment = load_environment(&config_dir.join("environment").join("standard.toml"))
                .map_err(|e| format!("Environment: {e}"))?;
            let races = load_race_presets(&config_dir.join("races").join("standard.toml"))
                .map_err(|e| format!("Races: {e}"))?;

            app.manage(Mutex::new(AppState {
                config_dir,
                profiles,
                environment,
                races,
            }));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_cars,
            load_car_config,
            save_car_config,
            get_race_presets,
            get_environment,
            save_environment,
            validate_car,
            resolve_and_detail,
            run_sim,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run app");
}
