# Crate

This crate provides a UI for [`rayna_engine`].

Through [`crate::integration`], the UI connects to the renderer (which runs on its own worker thread), sending messages back and forth (e.g. scene updates are sent to the worker, rendered frames are sent back to the UI).


# Features

- Multiple scene support, changed at runtime - currently these are loaded from [`rayna_engine::scene::preset::ALL()`], although this is not required for engine users.
- Multiple (debugging) render modes, showing UVs, normals, etc (see [`rayna_engine::render::render_opts::RenderMode`])
- Accumulated rendering (accumulate samples over time to improve quality)


# Crate Architecture

## Extensibility

I designed this crate to be semi-extensible, so the code in [`crate::backend`] is designed to be app-agnostic, with no specifics related to [`rayna_engine`].


# Cargo Features

## UI Backends
Currently this crate has two features, which enable different UI backends that can be used to render the application. (TODO) They can be selected at runtime by passing the `--backend="eframe"` CLI flag. 

- `backend_eframe`: Enables using [`eframe`] as a UI backend
- `backend_miniquad`: Enables using [`miniquad`] as a UI backend
